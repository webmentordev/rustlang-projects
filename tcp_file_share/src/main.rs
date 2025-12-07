use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = match TcpListener::bind("0.0.0.0:8888") {
        Ok(listener) => listener,
        Err(_) => TcpListener::bind("0.0.0.0:0").expect("Unable to assign any port."),
    };
    println!(
        "🚀 Listening at: http://127.0.0.1:{}",
        listener.local_addr()?.port()
    );
    for stream in listener.incoming().flatten() {
        thread::spawn(move || {
            if let Err(e) = handle_conn(stream) {
                println!("💀 Connectione error: {}", e);
            }
        });
    }
    Ok(())
}

fn handle_conn(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut filename = String::new();

    reader.read_line(&mut filename)?;
    let filename = filename.trim();
    println!("📃 File requsted: {}", filename);

    match File::open(filename) {
        Ok(mut file) => {
            let filesize = file.metadata()?.len();
            writeln!(stream, "{}", filesize)?;

            let mut buffer = [0; 4096];
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                stream.write_all(&buffer[..n])?;
            }
            println!("✅ File has been sent!");
        }
        Err(_) => {
            writeln!(stream, "0")?;
            println!("💀 File: {} not found!", filename);
        }
    }
    Ok(())
}
