use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("0.0.0.0:8888")?;
    let filename = "file.txt";
    writeln!(stream, "{}", filename)?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut filesize = String::new();
    reader.read_line(&mut filesize)?;
    let filesize: u64 = filesize.trim().parse()?;

    if filesize == 0 {
        println!("💀 Server: File not found!");
        return Ok(());
    }

    println!("🚀 Requested filesize: {} bytes", filesize);

    let mut received: u64 = 0;
    let mut buffer = [0; 4096];
    let mut newfile = File::create(format!("downloaded_{}", filename))?;
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        newfile.write_all(&buffer[..n])?;
        received += 1 as u64;
        if received >= filesize {
            break;
        }
    }
    println!("👌 File downloaded");
    Ok(())
}
