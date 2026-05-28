use jpeg_decoder::Decoder;
use rscam::Camera;
use std::io::Read;
use std::io::{Cursor, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const PORT: &str = "7765";

fn main() {
    let frame_buffer = Arc::new(Mutex::new(vec![]));
    let frame_buffer_clone = frame_buffer.clone();
    thread::spawn(move || {
        capture_frames(frame_buffer_clone);
    });
    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", PORT)).expect("Failed to bind to port 7765");

    println!("Server running at http://0.0.0.0:{}", PORT);
    println!("Visit http://192.168.140:{} in your browser", PORT);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let frame_buffer = frame_buffer.clone();
                thread::spawn(move || {
                    handle_client(stream, frame_buffer);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn capture_frames(frame_buffer: Arc<Mutex<Vec<u8>>>) {
    let mut camera = Camera::new("/dev/video0").expect("Failed to open camera");

    camera
        .start(&rscam::Config {
            interval: (1, 30),
            resolution: (WIDTH as u32, HEIGHT as u32),
            format: &*b"MJPG",
            ..Default::default()
        })
        .expect("Failed to start camera");

    loop {
        if let Ok(frame) = camera.capture() {
            let mut buffer = frame_buffer.lock().unwrap();
            buffer.clear();
            buffer.extend_from_slice(&frame[..]);
        }
    }
}

fn handle_client(mut stream: TcpStream, frame_buffer: Arc<Mutex<Vec<u8>>>) {
    let mut buffer = [0; 512];
    if stream.read(&mut buffer).is_err() {
        return;
    }
    let request = String::from_utf8_lossy(&buffer);
    if request.contains("GET / ") {
        let html = include_str!("../index.html");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(response.as_bytes());
    } else if request.contains("GET /stream ") {
        let response =
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        loop {
            let frame = frame_buffer.lock().unwrap().clone();
            if !frame.is_empty() {
                let boundary = "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: ";
                let header = format!("{}{}\r\n\r\n", boundary, frame.len());
                let footer = "\r\n";
                if stream.write_all(header.as_bytes()).is_err()
                    || stream.write_all(&frame).is_err()
                    || stream.write_all(footer.as_bytes()).is_err()
                {
                    break;
                }
            }
            thread::sleep(std::time::Duration::from_millis(33));
        }
    }
}
