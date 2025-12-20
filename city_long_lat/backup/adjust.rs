use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line_result) = lines.next() {
        let line = line_result?;
        let items: Vec<&str> = line.split(" ").collect();
        let last = items.last().unwrap();

        let newline = match last.parse::<f32>() {
            Ok(_) => line.to_string(),
            Err(_) => {
                if let Some(next_line_result) = lines.next() {
                    let next_line = next_line_result?;
                    format!("{} {}", line, next_line)
                } else {
                    line.to_string()
                }
            }
        };
        println!("{}", newline);
    }
    Ok(())
}
