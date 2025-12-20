use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "build" => {
                let bytes = std::fs::read("City-country-latitude-longitude-query.pdf").unwrap();
                let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();

                let lines: Vec<&str> = out.lines().collect();
                let mut result = String::new();
                let mut i = 0;

                while i < lines.len() {
                    let line = lines[i].trim();

                    if line.is_empty()
                        || line.contains("www.jinyi-solar.com")
                        || line.contains("City Province/State Country")
                    {
                        i += 1;
                        continue;
                    }
                    if i + 1 < lines.len() {
                        let next_line = lines[i + 1].trim();
                        if !next_line.is_empty()
                            && next_line
                                .chars()
                                .next()
                                .map_or(false, |c| c.is_numeric() || c == '-')
                        {
                            result.push_str(line);
                            result.push(' ');
                            result.push_str(next_line);
                            result.push('\n');
                            i += 2;
                            continue;
                        }
                    }
                    result.push_str(line);
                    result.push('\n');
                    i += 1;
                }

                fs::write("input.txt", result)?;

                let file = File::open("input.txt")?;
                let reader = BufReader::new(file);
                let mut lines = reader.lines();
                let mut processed = String::new();

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
                    processed.push_str(&newline);
                    processed.push('\n');
                }

                fs::write("result_file.txt", processed)?;
                fs::remove_file("input.txt")?;
                println!("🚀 Database is ready!");
            }
            _ => {
                let file = File::open("result_file.txt")?;
                let reader = BufReader::new(file);
                let lines = reader.lines();
                let mut found = false;

                for line_result in lines {
                    let line = line_result?;
                    if line.contains(&args[1]) {
                        let parts: Vec<&str> = line.split(" ").collect();
                        let size = parts.len();
                        let lng = parts[size - 1];
                        let lat = parts[size - 2];
                        found = true;
                        println!("Lng: {} & Lat: {}", lng, lat);
                    }
                }
                if found == false {
                    println!("☠️ Name is not found in database");
                }
            }
        }
    } else {
        println!("\nUsage:");
        println!("   * build           - Build the files woth info.");
        println!("   * <city-name>     - Cityname to find Longititude and latitude");
    }
    Ok(())
}
