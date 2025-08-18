use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};

fn main() {
    let listener = match TcpListener::bind("0.0.0.0:6341") {
        Ok(listener) => listener,
        Err(_) => TcpListener::bind("0.0.0.0:0").expect("Could not assign any port."),
    };
    let port = listener.local_addr().unwrap().port();
    println!("TCP Server listening: http://127.0.0.1:{}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            eprintln!("Error reading request: {}", e);
            return;
        }
        None => {
            eprintln!("No request line received");
            return;
        }
    };
    println!("{}", request);
    let (file, code, status) = match request.as_str() {
        "GET / HTTP/1.1" => ("index.html", 200, "OK"),
        "GET /contact HTTP/1.1" => ("contact.html", 200, "OK"),
        _ => ("404.html", 404, "NOT FOUND"),
    };

    let contents = fs::read_to_string(file).unwrap();
    let length = contents.len();

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type:text/html\r\nContent-Length: {}\r\n\r\n{}",
        code, status, length, contents
    );
    stream.write_all(response.as_bytes()).unwrap();
}
