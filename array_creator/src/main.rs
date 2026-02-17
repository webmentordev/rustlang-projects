use std::env;
use std::fs;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_path = env::current_dir()?;
    let mut files = vec![];

    println!("Which format would you like to print in? json, js, php");
    let mut convert_type = String::new();
    io::stdin()
        .read_line(&mut convert_type)
        .expect("Return data type is required. json, js, php");
    let convert_type = convert_type.trim();

    for entry in fs::read_dir(&current_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_stem() {
                files.push(name.to_string_lossy().into_owned());
            }
        }
    }

    match convert_type {
        "json" | "php" => print_json(&files),
        _ => print_js(&files),
    }

    Ok(())
}

fn print_json(array: &[String]) {
    println!("[");
    for (i, item) in array.iter().enumerate() {
        if i < array.len() - 1 {
            println!(r#"  "{}","#, item);
        } else {
            println!(r#"  "{}""#, item);
        }
    }
    println!("]");
}

fn print_js(array: &[String]) {
    println!("{{");
    for (i, item) in array.iter().enumerate() {
        if i < array.len() - 1 {
            println!(r#"  "{}","#, item);
        } else {
            println!(r#"  "{}""#, item);
        }
    }
    println!("}}");
}
