use std::fs::File;
use std::io::{BufReader, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("text.txt")?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        out.write_all(&buffer[..n])?;
    }

    Ok(())
}
