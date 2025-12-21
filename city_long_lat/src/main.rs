use actix_web::{App, HttpServer, Responder, Result, get, post, web};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

#[derive(Serialize)]
struct Response {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    records: Option<Vec<Address>>,
}

#[derive(Serialize, Clone)]
struct Address{
    location: String,
    longitude: f32,
    latitude: f32
}


#[derive(Deserialize)]
struct RequestBody {
    name: String,
}

struct AppData {
    city_data: Vec<Address>,
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

fn load_into_memory() -> Vec<Address>{
    let file = File::open("result_file.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let mut data: Vec<Address> = vec![];
    for line_result in lines.flatten() {
        let parts: Vec<&str> = line_result.split(" ").collect();
        let size = parts.len();
        let lng: f32 = parts[size - 1].parse().unwrap_or(0.0);
        let lat: f32 = parts[size - 2].parse().unwrap_or(0.0);
        data.push(Address{
            location: line_result,
            longitude: lng,
            latitude: lat
        });
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
async fn search(data: web::Data<AppData>, body: web::Json<RequestBody>) -> Result<impl Responder> {
    let lines = data.city_data.clone();
    let mut result_data: Vec<Address> = vec![];
    let mut found = false;

    let name_to_search = &body.name.to_lowercase();
    for address in lines {
        let line_search = address.location.to_lowercase();
        if line_search.contains(name_to_search) {
            let parts: Vec<&str> = address.location.split(" ").collect();
            if parts.len() > 2 {
                found = true;
                result_data.push(address);
            }
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
        records: Some(result_data),
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
        let mut line = line_result?;
        loop {
            let items: Vec<&str> = line.split(" ").collect();
            let last = items.last().unwrap();
            match last.parse::<f64>() {
                Ok(_) => {
                    break;
                }
                Err(_) => {
                    if let Some(next_line_result) = lines.next() {
                        let next_line = next_line_result?;
                        line = format!("{} {}", line, next_line);
                    } else {
                        break;
                    }
                }
            }
        }
        processed.push_str(&line);
        processed.push('\n');
    }

    fs::write("result_file.txt", processed)?;
    fs::remove_file("input.txt")?;
    println!("🚀 Database has been created!");
    Ok(())
}
