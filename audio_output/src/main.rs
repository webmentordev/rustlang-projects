use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use nnnoiseless::DenoiseState;
use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const VIRTUAL_SINK_NAME: &str = "RustCustomOutput";
const VIRTUAL_SINK_DESC: &str = "RustCustomOutput (Denoised Mic)";
const FRAME_SIZE: usize = DenoiseState::FRAME_SIZE;

#[derive(Parser)]
#[command(name = "virtual-audio", about = "Noise-suppressed virtual mic")]
struct Args {
    #[arg(
        long,
        default_value_t = 0.5,
        help = "Voice activity threshold (0.0–1.0)"
    )]
    threshold: f32,

    #[arg(
        long,
        default_value_t = 0.02,
        help = "Noise attenuation when suppressed (0.0=mute, 1.0=keep)"
    )]
    attenuation: f32,
}

fn pactl(args: &[&str]) -> Result<String, String> {
    let out = Command::new("pactl")
        .args(args)
        .output()
        .map_err(|e| format!("pactl error: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn load_null_sink() -> Result<u32, String> {
    pactl(&[
        "load-module",
        "module-null-sink",
        &format!("sink_name={VIRTUAL_SINK_NAME}"),
        &format!("sink_properties=device.description={VIRTUAL_SINK_DESC}"),
        "rate=48000",
        "channels=1",
    ])?
    .parse::<u32>()
    .map_err(|_| "unexpected module id".into())
}

fn unload(id: u32) {
    let _ = pactl(&["unload-module", &id.to_string()]);
}

fn silence_alsa_errors() {
    unsafe {
        let devnull = libc::open(
            b"/dev/null\0".as_ptr() as *const libc::c_char,
            libc::O_WRONLY,
        );
        if devnull >= 0 {
            libc::dup2(devnull, libc::STDERR_FILENO);
            libc::close(devnull);
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.threshold < 0.0 || args.threshold > 1.0 {
        eprintln!("--threshold must be 0.0–1.0");
        std::process::exit(1);
    }
    if args.attenuation < 0.0 || args.attenuation > 1.0 {
        eprintln!("--attenuation must be 0.0–1.0");
        std::process::exit(1);
    }

    let sink_id = match load_null_sink() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Error creating virtual sink: {e}");
            eprintln!("Is PipeWire running? Try: systemctl --user status pipewire");
            std::process::exit(1);
        }
    };

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Silence ALSA spam after we're done with pactl
    silence_alsa_errors();

    let host = cpal::default_host();

    let input_device = host
        .input_devices()
        .unwrap()
        .find(|d| d.name().unwrap_or_default() == "pipewire")
        .or_else(|| host.default_input_device())
        .unwrap_or_else(|| {
            eprintln!("No input device found.");
            unload(sink_id);
            std::process::exit(1);
        });

    let output_device = host
        .output_devices()
        .unwrap()
        .find(|d| d.name().unwrap_or_default() == "pipewire")
        .or_else(|| host.default_output_device())
        .unwrap_or_else(|| {
            eprintln!("No output device found.");
            unload(sink_id);
            std::process::exit(1);
        });

    let in_config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
    let out_config: cpal::StreamConfig = output_device.default_output_config().unwrap().into();

    let threshold = args.threshold;
    let attenuation = args.attenuation;

    let ring: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let ring_in = ring.clone();
    let ring_out = ring.clone();

    let leftover: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let denoise = Arc::new(Mutex::new(DenoiseState::new()));

    let input_stream = input_device
        .build_input_stream(
            &in_config,
            move |data: &[f32], _| {
                let mut lo = leftover.lock().unwrap();
                lo.extend_from_slice(data);

                let mut denoiser = denoise.lock().unwrap();
                let mut processed: Vec<f32> = Vec::new();

                while lo.len() >= FRAME_SIZE {
                    let frame: Vec<f32> = lo.drain(..FRAME_SIZE).collect();
                    let scaled: Vec<f32> = frame.iter().map(|&s| s * 32768.0).collect();
                    let mut out_frame = vec![0.0f32; FRAME_SIZE];

                    let vad = denoiser.process_frame(&mut out_frame, &scaled);

                    for (i, s) in out_frame.iter().enumerate() {
                        let clean = s / 32768.0;
                        let result = if vad < threshold {
                            frame[i] * attenuation
                        } else {
                            let blend =
                                ((vad - threshold) / (1.0 - threshold + 1e-6)).clamp(0.0, 1.0);
                            clean * blend + frame[i] * (1.0 - blend)
                        };
                        processed.push(result);
                    }
                }

                ring_in.lock().unwrap().extend_from_slice(&processed);
            },
            |e| eprintln!("Input error: {e}"),
            None,
        )
        .unwrap();

    let output_stream = output_device
        .build_output_stream(
            &out_config,
            move |data: &mut [f32], _| {
                let mut buf = ring_out.lock().unwrap();
                let len = data.len().min(buf.len());
                data[..len].copy_from_slice(&buf[..len]);
                buf.drain(..len);
                for s in &mut data[len..] {
                    *s = 0.0;
                }
            },
            |e| eprintln!("Output error: {e}"),
            None,
        )
        .unwrap();

    // Restore stderr for our own prints
    unsafe {
        let tty = libc::open(
            b"/dev/tty\0".as_ptr() as *const libc::c_char,
            libc::O_WRONLY,
        );
        if tty >= 0 {
            libc::dup2(tty, libc::STDERR_FILENO);
            libc::close(tty);
        }
    }

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    println!();
    println!("┌──────────────────────────────────────────────┐");
    println!("│      RustCustomOutput — Active               │");
    println!("├──────────────────────────────────────────────┤");
    println!("│  Threshold  : {:<31}│", threshold);
    println!("│  Attenuation: {:<31}│", attenuation);
    println!("├──────────────────────────────────────────────┤");
    println!("│  Set mic input in Discord/Zoom/any app to:   │");
    println!("│    \"RustCustomOutput (Denoised Mic)\"         │");
    println!("└──────────────────────────────────────────────┘");
    println!();
    println!("Press Ctrl+C or Enter to stop.");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();
    let r2 = running.clone();
    std::thread::spawn(move || {
        let mut s = String::new();
        let _ = io::stdin().read_line(&mut s);
        r2.store(false, Ordering::SeqCst);
    });

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    print!("\nCleaning up... ");
    io::stdout().flush().ok();
    drop(input_stream);
    drop(output_stream);
    unload(sink_id);
    println!("done.");
}
