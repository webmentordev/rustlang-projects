use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7688").unwrap();
    println!(
        "Listening on port {}...",
        listener.local_addr().unwrap().port()
    );

    for mut stream in listener.incoming().flatten() {
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            stream.read(&mut buffer).unwrap();

            let response_headers = "HTTP/1.1 200 OK\r\n\
                Content-Type: text/event-stream\r\n\
                Cache-Control: no-cache\r\n\
                Access-Control-Allow-Origin: *\r\n\
                \r\n";

            stream.write_all(response_headers.as_bytes()).unwrap();

            let array = vec!["Hi", "there", "mate"];
            for item in array {
                thread::sleep(Duration::from_secs(2));

                let message = format!("data: {}\n\n", item);
                if stream.write_all(message.as_bytes()).is_err() {
                    break;
                }
            }
            stream.write_all(b"event: done\ndata: \n\n").unwrap();
        });
    }
}
