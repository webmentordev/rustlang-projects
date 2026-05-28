use jpeg_decoder::Decoder;
use minifb::{Window, WindowOptions};
use rscam::Camera;
use std::io::Cursor;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() {
    let mut window = Window::new("Webcam", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Failed to create window");

    let mut camera = Camera::new("/dev/video0").expect("Failed to open camera");

    camera
        .start(&rscam::Config {
            interval: (1, 30),
            resolution: (WIDTH as u32, HEIGHT as u32),
            format: &*b"MJPG",
            ..Default::default()
        })
        .expect("Failed to start camera");

    let mut buffer = vec![0u32; WIDTH * HEIGHT];

    while window.is_open() {
        if let Ok(frame) = camera.capture() {
            let mut decoder = Decoder::new(Cursor::new(&frame[..]));
            if let Ok(pixels) = decoder.decode() {
                for i in 0..pixels.len() / 3 {
                    let r = pixels[i * 3] as u32;
                    let g = pixels[i * 3 + 1] as u32;
                    let b = pixels[i * 3 + 2] as u32;
                    buffer[i] = (r << 16) | (g << 8) | b;
                }

                window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            }
        }
    }
}
