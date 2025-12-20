use actix_web::{App, HttpServer, Responder, Result, get, post, web};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

#[derive(Serialize)]
struct Response {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    records: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RequestBody {
    name: String,
}

struct AppData {
    city_data: Vec<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if File::open("result_file.txt").is_ok() {
        println!("✅ Database found.");
    }else{
        if let Err(e) = handle_build() {
            eprintln!("Build error: {}", e);
            return Ok(());
        }
    }
    
    println!("🚀 Listening at: http://0.0.0.1:8030");
    HttpServer::new(|| App::new().app_data(web::Data::new(AppData {
                city_data: load_into_memory(),
            })).service(search).service(index).service(get_all))
        .bind(("0.0.0.0", 8030))?
        .run()
        .await
}

fn load_into_memory() -> Vec<String>{
    let file = File::open("result_file.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let mut data: Vec<String> = vec![];
    for line_result in lines.flatten() {
        data.push(line_result);
    }
    data
}

#[get("/")]
async fn index() -> Result<impl Responder> {
    Ok(web::Json(Response {
        success: true,
        message: "Welcome to City Coordinated finder Free API".to_string(),
        records: None,
    }))
}

#[post("/search")]
async fn search(body: web::Json<RequestBody>) -> Result<impl Responder> {
    let file = File::open("result_file.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let mut data: Vec<String> = vec![];
    let mut found = false;

    for line_result in lines {
        let line = line_result.unwrap();
        if line.contains(&body.name) {
            let parts: Vec<&str> = line.split(" ").collect();
            let size = parts.len();
            let lng = parts[size - 1];
            let lat = parts[size - 2];
            found = true;
            data.push(format!("Lng: {} & Lat: {}", lng, lat));
        }
    }
    if found == false {
        return Ok(web::Json(Response {
            success: false,
            message: "☠️ Name is not found in database".to_string(),
            records: None,
        }));
    }
    Ok(web::Json(Response {
        success: true,
        message: "City data found!".to_string(),
        records: Some(data),
    }))
}

#[get("/get-all")]
async fn get_all(data: web::Data<AppData>) -> Result<impl Responder> {
    Ok(web::Json(Response {
        success: true,
        message: "Fetched whole database".to_string(),
        records: Some(data.city_data.clone()),
    }))
}

fn handle_build() -> Result<(), Box<dyn std::error::Error>> {
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
    println!("🚀 Database has been created!");
    Ok(())
}
