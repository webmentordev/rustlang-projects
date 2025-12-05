use chrono::{Timelike, Utc};
use glob::glob;
use regex::Regex;
use reqwest::blocking::multipart;
use rusqlite::{Connection, Result as SqliteResult};
use rust_xlsxwriter::{Workbook, XlsxError};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct Config {
    api: String,
    url: HashMap<String, String>,
}

#[derive(Debug)]
struct PriceInfo {
    price_numeric: String,
    auction_date: String,
    deadline_date: String,
    negotiation: String,
}

#[derive(Debug)]
struct PropertyFeatures {
    bedrooms: String,
    bathrooms: String,
    parking: String,
    lounges: String,
    floor_area: String,
    land_area: String,
}

#[derive(Debug)]
struct Listing {
    id: String,
    agent_name: String,
    agent_number: String,
    title: String,
    subtitle: String,
    address: String,
    suburb: String,
    region: String,
    price: String,
    auction_date: String,
    deadline_date: String,
    negotiation: String,
    link: String,
    previous_price: String,
    features: PropertyFeatures,
    listing_type: String,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("info.json")?;
    let config: Config = serde_json::from_str(&config_content)?;
    Ok(config)
}

fn init_database() -> SqliteResult<Connection> {
    let conn = Connection::open("listings.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS listings (
            id TEXT PRIMARY KEY,
            price TEXT,
            auction_date TEXT,
            deadline_date TEXT,
            negotiation TEXT,
            previous_price TEXT,
            last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    Ok(conn)
}

fn get_existing_price(conn: &Connection, id: &str) -> SqliteResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT price FROM listings WHERE id = ?")?;
    let result = stmt.query_row([id], |row| row.get(0));

    match result {
        Ok(price) => Ok(Some(price)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn upsert_listing(conn: &Connection, listing: &Listing) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO listings (
            id, price, auction_date, deadline_date, negotiation,
            previous_price, last_updated
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            price = excluded.price,
            auction_date = excluded.auction_date,
            deadline_date = excluded.deadline_date,
            negotiation = excluded.negotiation,
            previous_price = CASE
                WHEN listings.price != excluded.price AND listings.price != ''
                THEN listings.price
                ELSE listings.previous_price
            END,
            last_updated = CURRENT_TIMESTAMP",
        [
            &listing.id,
            &listing.price,
            &listing.auction_date,
            &listing.deadline_date,
            &listing.negotiation,
            &listing.previous_price,
        ],
    )?;
    Ok(())
}

fn get_final_listing(conn: &Connection, id: &str) -> SqliteResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT previous_price FROM listings WHERE id = ?")?;
    let result = stmt.query_row([id], |row| row.get(0));

    match result {
        Ok(prev_price) => Ok(Some(prev_price)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn extract_property_features(details: &scraper::ElementRef) -> PropertyFeatures {
    let mut features = PropertyFeatures {
        bedrooms: String::new(),
        bathrooms: String::new(),
        parking: String::new(),
        lounges: String::new(),
        floor_area: String::new(),
        land_area: String::new(),
    };

    let features_selector =
        Selector::parse(".tm-property-search-card-attribute-icons__features").unwrap();
    let metric_selector =
        Selector::parse(".tm-property-search-card-attribute-icons__metric").unwrap();

    if let Some(features_container) = details.select(&features_selector).next() {
        for metric in features_container.select(&metric_selector) {
            let icon_selector = Selector::parse("tg-icon").unwrap();

            if let Some(icon) = metric.select(&icon_selector).next() {
                let feature_type = icon.value().attr("alt").unwrap_or("");

                let value_selector =
                    Selector::parse(".tm-property-search-card-attribute-icons__metric-value")
                        .unwrap();
                let value = metric
                    .select(&value_selector)
                    .next()
                    .map(|e| clean_text(&e.text().collect::<Vec<_>>().join("")))
                    .unwrap_or_default();

                match feature_type {
                    "Bedrooms" => features.bedrooms = value,
                    "Bathrooms" => features.bathrooms = value,
                    "Total parking" => features.parking = value,
                    "Living areas/Lounges" => features.lounges = value,
                    "Floor area" => features.floor_area = value,
                    "Land area" => features.land_area = value,
                    _ => {}
                }
            }
        }
    }

    features
}

fn upload_to_pocketbase(api_url: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Uploading file to PocketBase...");

    let file = fs::read(file_path)?;
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("listing.xlsx");

    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(file)
            .file_name(file_name.to_string())
            .mime_str("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")?,
    );

    let url = format!("{}/api/collections/files/records", api_url);
    let client = reqwest::blocking::Client::new();
    let response = client.post(&url).multipart(form).send()?;

    if response.status().is_success() {
        println!("File uploaded successfully to PocketBase!");
        println!("Response: {}", response.text()?);
    } else {
        println!("Upload failed with status: {}", response.status());
        println!("Response: {}", response.text()?);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let config = load_config()?;

    let conn = init_database()?;
    let mut results = Vec::new();

    for (listing_type, _url) in &config.url {
        let pattern = format!("pages/{}_page_*.html", listing_type);
        println!(
            "Processing {} listings from pattern: {}",
            listing_type, pattern
        );

        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                let html = fs::read_to_string(&path).expect("Cannot read HTML file");
                let document = Html::parse_document(&html);

                let outer_selector =
                    Selector::parse(".tm-property-premium-listing-card__link").unwrap();
                let details_selector =
                    Selector::parse(".tm-property-premium-listing-card__details-container")
                        .unwrap();
                let agent_selector =
                    Selector::parse(".tm-property-premium-listing-card__agents-name").unwrap();
                let agent_number_selector =
                    Selector::parse(r#"[tmid="premium-listing-card-agent-details"]"#).unwrap();
                let price_selector =
                    Selector::parse(".tm-property-search-card-price-attribute__price").unwrap();
                let title_selector =
                    Selector::parse(r#"[tmid="premium-listing-card-title"]"#).unwrap();
                let subtitle_selector =
                    Selector::parse(r#"[tmid="premium-listing-card-subtitle"]"#).unwrap();

                for outer in document.select(&outer_selector) {
                    let link = outer.value().attr("href").unwrap_or("").to_string();
                    let unique_id = link
                        .split("/listing/")
                        .nth(1)
                        .and_then(|s| s.split('?').next())
                        .unwrap_or("")
                        .to_string();

                    if let Some(details) = outer.select(&details_selector).next() {
                        let agent_name = details
                            .select(&agent_selector)
                            .next()
                            .map(|e| clean_text(&e.text().collect::<Vec<_>>().join("")))
                            .unwrap_or_default();

                        let mut agent_numbers = details
                            .select(&agent_number_selector)
                            .map(|e| clean_text(&e.text().collect::<Vec<_>>().join("")))
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>();

                        if !agent_numbers.is_empty() {
                            agent_numbers.remove(0);
                        }

                        let agent_number = agent_numbers.join(", ");

                        let title = details
                            .select(&title_selector)
                            .next()
                            .map(|e| clean_text(&e.text().collect::<Vec<_>>().join("")))
                            .unwrap_or_default();

                        let sub = details
                            .select(&subtitle_selector)
                            .next()
                            .map(|e| clean_text(&e.text().collect::<Vec<_>>().join("")))
                            .unwrap_or_default();

                        let (address, suburb, region) = parse_location(&sub);

                        let price_raw = details
                            .select(&price_selector)
                            .next()
                            .map(|e| clean_text(&e.text().collect::<Vec<_>>().join("")))
                            .unwrap_or_default();

                        let price_info = parse_price(&price_raw);
                        let previous_price =
                            get_existing_price(&conn, &unique_id)?.unwrap_or_default();

                        let features = extract_property_features(&details);

                        let listing = Listing {
                            id: unique_id.clone(),
                            agent_name,
                            agent_number,
                            title,
                            subtitle: sub,
                            address,
                            suburb,
                            region,
                            price: price_info.price_numeric.clone(),
                            auction_date: price_info.auction_date.clone(),
                            deadline_date: price_info.deadline_date.clone(),
                            negotiation: price_info.negotiation.clone(),
                            link: link.clone(),
                            previous_price,
                            features,
                            listing_type: listing_type.clone(),
                        };

                        upsert_listing(&conn, &listing)?;

                        let final_previous_price =
                            get_final_listing(&conn, &unique_id)?.unwrap_or_default();

                        let final_listing = Listing {
                            previous_price: final_previous_price,
                            ..listing
                        };

                        results.push(final_listing);
                    }
                }
            }
        }
    }

    let excel_filename = export_to_excel(&results)?;

    if let Err(e) = upload_to_pocketbase(&config.api, &excel_filename) {
        eprintln!("Failed to upload to PocketBase: {}", e);
    }

    let duration = start_time.elapsed();
    println!("Data saved to database and Excel file");
    println!("Total execution time: {:.2?}", duration);
    Ok(())
}

fn export_to_excel(results: &[Listing]) -> Result<String, XlsxError> {
    let now = Utc::now();
    let hour = now.hour();
    let am_pm = if hour >= 12 { "PM" } else { "AM" };
    let hour12 = match hour % 12 {
        0 => 12,
        h => h,
    };

    fs::create_dir_all("sheets").expect("Failed to create sheets directory");

    let minute = now.minute();
    let filename = format!(
        "sheets/listings_{}_{}{}_{:02}_UTC.xlsx",
        now.format("%Y-%m-%d"),
        hour12,
        am_pm,
        minute
    );

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let headers = [
        "Unique ID",
        "Listing Type",
        "Agent Name",
        "Agent Number",
        "Title",
        "Subtitle",
        "Address",
        "Suburb",
        "Region",
        "Price",
        "Previous Price",
        "Auction Date",
        "Deadline Date",
        "Price by Negotiation",
        "Bedrooms",
        "Bathrooms",
        "Parking",
        "Floor Area",
        "Lounges",
        "Land Area",
        "Link",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col as u16, &header.to_string())?;
    }

    for (row, listing) in results.iter().enumerate() {
        let row_index = (row + 1) as u32;
        worksheet.write_string(row_index, 0, &listing.id)?;
        worksheet.write_string(row_index, 1, &listing.listing_type)?;
        worksheet.write_string(row_index, 2, &listing.agent_name)?;
        worksheet.write_string(row_index, 3, &listing.agent_number)?;
        worksheet.write_string(row_index, 4, &listing.title)?;
        worksheet.write_string(row_index, 5, &listing.subtitle)?;
        worksheet.write_string(row_index, 6, &listing.address)?;
        worksheet.write_string(row_index, 7, &listing.suburb)?;
        worksheet.write_string(row_index, 8, &listing.region)?;
        worksheet.write_string(row_index, 9, &listing.price)?;
        worksheet.write_string(row_index, 10, &listing.previous_price)?;
        worksheet.write_string(row_index, 11, &listing.auction_date)?;
        worksheet.write_string(row_index, 12, &listing.deadline_date)?;
        worksheet.write_string(row_index, 13, &listing.negotiation)?;
        worksheet.write_string(row_index, 14, &listing.features.bedrooms)?;
        worksheet.write_string(row_index, 15, &listing.features.bathrooms)?;
        worksheet.write_string(row_index, 16, &listing.features.parking)?;
        worksheet.write_string(row_index, 17, &listing.features.floor_area)?;
        worksheet.write_string(row_index, 18, &listing.features.lounges)?;
        worksheet.write_string(row_index, 19, &listing.features.land_area)?;
        worksheet.write_string(row_index, 20, &listing.link)?;
    }

    workbook.save(&filename)?;
    println!("Excel file saved to {}", filename);
    Ok(filename)
}

fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_price(price_text: &str) -> PriceInfo {
    let mut info = PriceInfo {
        price_numeric: String::new(),
        auction_date: String::new(),
        deadline_date: String::new(),
        negotiation: String::new(),
    };

    if price_text.to_lowercase().contains("price by negotiation") {
        info.negotiation = "Yes".to_string();
        return info;
    }

    if price_text.to_lowercase().contains("auction") {
        info.auction_date = price_text.to_string();
        return info;
    }

    if price_text.to_lowercase().contains("deadline sale") {
        info.deadline_date = price_text.to_string();
        return info;
    }

    let re = Regex::new(r"\$[\d,]+").unwrap();
    if let Some(mat) = re.find(price_text) {
        let price_str = mat.as_str();
        info.price_numeric = price_str.replace("$", "").replace(",", "");
    }

    info
}

fn parse_location(location: &str) -> (String, String, String) {
    let parts: Vec<&str> = location.split(',').map(|s| s.trim()).collect();

    match parts.len() {
        0 => (String::new(), String::new(), String::new()),
        1 => (parts[0].to_string(), String::new(), String::new()),
        2 => (parts[0].to_string(), parts[1].to_string(), String::new()),
        _ => (
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2..].join(", "),
        ),
    }
}
