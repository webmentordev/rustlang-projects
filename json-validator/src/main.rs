use std::env;
use std::fs;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage:\n valid_json <file-path.json>");
        return;
    }

    let file_name = &args[1];
    if !file_name.ends_with(".json") {
        println!("{} is not a json file", file_name);
        return;
    }

    let content = fs::read_to_string(file_name).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    });

    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(_) => println!("Json file is valid!"),
        Err(e) => println!("Invalid Json: {}", e),
    }
}
