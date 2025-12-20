use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file = File::open("result_file.txt")?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line_result) = lines.next() {
            let line = line_result?;
            if line.contains(&args[1]) {
                let parts: Vec<&str> = line.split(" ").collect();
                let size = parts.len();
                let lng = parts[size - 1];
                let lat = parts[size - 2];
                println!("Lng: {} & Lat: {}", lng, lat);
            }
        }
    } else {
        println!("\nUsage:");
        println!("   * <city-name>     - Cityname to find Longititude and latitude");
    }
    Ok(())
}
