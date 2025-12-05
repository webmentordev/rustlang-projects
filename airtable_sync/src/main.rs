use actix_web::{App, HttpResponse, HttpServer, web};
use rand::Rng;
use reqwest;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

const SYNC_INTERVAL_SECONDS: u64 = 7200;
const CHUNK_SIZE: usize = 300;

// Toggle field filtering: true = only send ALLOWED_FIELDS, false = send all fields
const ENABLE_FIELD_FILTERING: bool = true;

const ALLOWED_FIELDS: &[&str] = &[
    "Airtable Record ID",
    "Name",
    "IP Address",
    "Record ID",
    "Call Status",
    // Add or remove colum names
];

#[derive(Debug, Deserialize)]
struct AirtableResponse {
    records: Vec<RequestData>,
    #[serde(default)]
    offset: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RequestData {
    #[serde(rename = "createdTime")]
    created_time: String,
    fields: HashMap<String, Value>,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    slug: Option<String>,
}

#[derive(Debug, Serialize)]
struct PaginatedResponse {
    records: Vec<RequestData>,
    total: usize,
    page: usize,
    page_size: usize,
    total_pages: usize,
    remaining: usize,
}

#[derive(Debug, Serialize)]
struct SingleRecordResponse {
    record: RequestData,
    slug: String,
}

fn generate_slug(full_name: &str) -> String {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen_range(99..10000);

    let slug_base = full_name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    format!("{}-{}", slug_base, random_number)
}

fn filter_fields(fields: &HashMap<String, Value>) -> HashMap<String, Value> {
    if ENABLE_FIELD_FILTERING {
        fields
            .iter()
            .filter(|(key, _)| ALLOWED_FIELDS.contains(&key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        fields.clone()
    }
}

async fn sync_airtable_data(
    db: web::Data<Mutex<Connection>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let airtable_token =
        std::env::var("AIRTABLE_TOKEN").expect("AIRTABLE_TOKEN environment variable not set");
    let airtable_url =
        std::env::var("AIRTABLE_URL").expect("AIRTABLE_URL environment variable not set");

    let client = reqwest::Client::new();
    let mut all_records = Vec::new();
    let mut offset: Option<String> = None;
    loop {
        let mut url = format!("{}?pageSize=100", airtable_url);
        if let Some(ref offset_value) = offset {
            url = format!("{}&offset={}", url, offset_value);
        }
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", airtable_token))
            .send()
            .await?;
        let airtable_data: AirtableResponse = response.json().await?;
        all_records.extend(airtable_data.records);
        if let Some(next_offset) = airtable_data.offset {
            offset = Some(next_offset);
        } else {
            break;
        }
    }

    println!("Finished fetching. Total records: {}", all_records.len());

    let conn = db.lock().unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS airtable_records (
            id TEXT PRIMARY KEY,
            created_time TEXT NOT NULL,
            fields TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    for record in &all_records {
        let fields_json = serde_json::to_string(&record.fields)?;

        // Extract Column from fields and generate slug for uniqe data fetching
        let full_name = record
            .fields
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or("record");

        let slug = generate_slug(full_name);

        conn.execute(
            "INSERT OR REPLACE INTO airtable_records (id, created_time, fields, slug) VALUES (?1, ?2, ?3, ?4)",
            params![record.id, record.created_time, fields_json, slug],
        )?;
    }

    println!("Synced {} records to database", all_records.len());
    Ok(())
}

async fn get_records(
    db: web::Data<Mutex<Connection>>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let page = query
        .get("page")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);

    let page_size = query
        .get("page_size")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(CHUNK_SIZE);

    let offset = (page - 1) * page_size;

    let conn = match db.lock() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Database lock error: {}", e),
                "status": "error",
            }));
        }
    };
    let total: usize = match conn.query_row("SELECT COUNT(*) FROM airtable_records", [], |row| {
        row.get(0)
    }) {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Failed to count records: {}", e),
                "status": "error",
            }));
        }
    };
    let mut stmt = match conn
        .prepare("SELECT id, created_time, fields, slug FROM airtable_records LIMIT ?1 OFFSET ?2")
    {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Failed to prepare query: {}", e),
                "status": "error",
            }));
        }
    };

    let records_iter = match stmt.query_map(params![page_size, offset], |row| {
        let id: String = row.get(0)?;
        let created_time: String = row.get(1)?;
        let fields_json: String = row.get(2)?;
        let slug: String = row.get(3)?;
        let fields: HashMap<String, Value> =
            serde_json::from_str(&fields_json).unwrap_or_else(|_| HashMap::new());

        Ok((id, created_time, fields, slug))
    }) {
        Ok(iter) => iter,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Failed to query records: {}", e),
                "status": "error",
            }));
        }
    };

    let mut records = Vec::new();
    for record_result in records_iter {
        if let Ok((id, created_time, fields, slug)) = record_result {
            let filtered_fields = filter_fields(&fields);
            records.push(RequestData {
                id,
                created_time,
                fields: filtered_fields,
                slug: Some(slug),
            });
        }
    }

    let total_pages = (total + page_size - 1) / page_size;
    let remaining = if offset + records.len() < total {
        total - (offset + records.len())
    } else {
        0
    };

    HttpResponse::Ok().json(PaginatedResponse {
        records,
        total,
        page,
        page_size,
        total_pages,
        remaining,
    })
}

async fn get_single_record(
    db: web::Data<Mutex<Connection>>,
    slug: web::Path<String>,
) -> HttpResponse {
    let conn = match db.lock() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Database lock error: {}", e),
                "status": "error",
            }));
        }
    };

    let mut stmt = match conn
        .prepare("SELECT id, created_time, fields, slug FROM airtable_records WHERE slug = ?1")
    {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Failed to prepare query: {}", e),
                "status": "error",
            }));
        }
    };

    let result = stmt.query_row(params![slug.as_str()], |row| {
        let id: String = row.get(0)?;
        let created_time: String = row.get(1)?;
        let fields_json: String = row.get(2)?;
        let slug: String = row.get(3)?;
        let fields: HashMap<String, Value> =
            serde_json::from_str(&fields_json).unwrap_or_else(|_| HashMap::new());

        Ok((id, created_time, fields, slug))
    });

    match result {
        Ok((id, created_time, fields, slug)) => {
            let filtered_fields = filter_fields(&fields);
            HttpResponse::Ok().json(SingleRecordResponse {
                record: RequestData {
                    id,
                    created_time,
                    fields: filtered_fields,
                    slug: Some(slug.clone()),
                },
                slug,
            })
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "message": "Record not found",
                "status": "error",
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": format!("Failed to fetch record: {}", e),
            "status": "error",
        })),
    }
}

async fn start_background_sync(db: web::Data<Mutex<Connection>>) {
    actix_web::rt::spawn(async move {
        loop {
            println!("Starting Airtable sync...");
            match sync_airtable_data(db.clone()).await {
                Ok(_) => println!("Airtable sync completed successfully"),
                Err(e) => eprintln!("Airtable sync failed: {}", e),
            }

            let duration = Duration::from_secs(SYNC_INTERVAL_SECONDS);
            let hours: f64 = SYNC_INTERVAL_SECONDS as f64 / 3600.0;
            println!(
                "Next sync in {} seconds or {:.2} hours",
                SYNC_INTERVAL_SECONDS, hours
            );
            tokio::time::sleep(duration).await;
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let conn = Connection::open("records.db")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let db = web::Data::new(Mutex::new(conn));

    start_background_sync(db.clone()).await;

    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .route("/", web::get().to(index))
            .route("/records", web::get().to(get_records))
            .route("/record/{slug}", web::get().to(get_single_record))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Welcome to ATSyncer!")
}
