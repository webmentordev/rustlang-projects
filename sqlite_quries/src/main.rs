use std::sync::Mutex;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web, post};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::io::Result as ioResult;

struct AppData{
    database: Mutex<Connection>
}

#[derive(Deserialize)]
struct CreateRequest{
    name: String,
    age: u32
}

#[derive(Serialize)]
struct RecordsResponse{
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    records: Option<Vec<serde_json::Value>>
}

#[derive(Serialize)]
struct SingleRecord{
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<serde_json::Value>
}


#[actix_web::main]
async fn main() -> ioResult<()>{
    let port = 8888;
    let conn = Connection::open("databases.sqlite").unwrap();
    conn.execute("
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        age BIGINT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )", []).unwrap();
    let app_data = web::Data::new(AppData{
        database: Mutex::new(conn)
    });
    println!("Listening at: http://127.0.0.1:{}", port);
    HttpServer::new(move || {
        App::new().app_data(app_data.clone()).service(hello).service(create_user).service(records).service(get_record).service(delete_record)
    }).bind(("127.0.0.1", port))?.run().await
}

// Just for info
#[get("/")]
async fn hello() -> impl Responder{
    HttpResponse::Ok().json(RecordsResponse{
        message: "Successfulyl fetched!".to_string(),
        success: true,
        records: None
    })
}

// Add a new record / user
#[post("/create")]
async fn create_user(db: web::Data<AppData>, request: web::Json<CreateRequest>) -> ioResult<impl Responder>{
    {
        let db = db.database.lock().unwrap();
        if let Err(e) = db.execute("INSERT into users (name, age) VALUES (?1, ?2)", 
            [request.name.to_string(), request.age.to_string()]
        ){
            return Ok(HttpResponse::InternalServerError().json(RecordsResponse{
                message: format!("Error: {}", e),
                success: false,
                records: None
            }))
        }
    }
    Ok(HttpResponse::Ok().json(RecordsResponse{
        message: "User has been created!".to_string(),
        success: true,
        records: None
    }))
}

// Get multiple records
#[get("/records")]
async fn records(db: web::Data<AppData>) -> ioResult<impl Responder>{
    let db = db.database.lock().unwrap();
    let mut stmt = db.prepare("SELECT name, age from users").unwrap();
    let users: Result<Vec<_>, _> = 
    stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "name": row.get::<_, String>(0)?,
            "age": row.get::<_, u32>(1)?
        }))
    }).and_then(|mapped_rows| mapped_rows.collect());

    match users {
        Ok(records) => Ok(HttpResponse::Ok().json(RecordsResponse {
            success: true,
            message: "Records retrieved successfully".to_string(),
            records: Some(records),
        })),
        Err(err) => Ok(HttpResponse::InternalServerError().json(RecordsResponse {
            success: false,
            message: format!("Failed to fetch records: {}", err),
            records: None,
        }))
    }
}


// Get a single record
#[get("/record/{id}")]
async fn get_record(db: web::Data<AppData>, params: web::Path<i32>) -> ioResult<impl Responder>{
    let id = params.into_inner();
    let db = db.database.lock().unwrap();
    let record = db.query_row("SELECT name, age from users WHERE id = ?1 LIMIT 1", 
    [id], 
    |row| {
        Ok(serde_json::json!({
            "name": row.get::<_, String>(0)?,
            "age": row.get::<_, u32>(1)?
        }))
    });

    match record {
        Ok(data) => Ok(HttpResponse::Ok().json(SingleRecord {
            success: true,
            message: "Record retrieved successfully".to_string(),
            record: Some(data),
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok(HttpResponse::NotFound().json(SingleRecord {
                success: false,
                message: format!("No record found with id: {}", id),
                record: None,
            }))
        },
        Err(err) => Ok(HttpResponse::InternalServerError().json(SingleRecord {
            success: false,
            message: format!("Database error: {}", err),
            record: None,
        }))
    }
}


// Delete a record
#[post("/delete-record/{id}")]
async fn delete_record(db: web::Data<AppData>, params: web::Path<i32>) -> ioResult<impl Responder> {
    let id = params.into_inner();
    let db = db.database.lock().unwrap();
    
    let result = db.execute("DELETE FROM users WHERE id = ?1", [id]);

    match result {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": format!("Record with id {} deleted successfully", id),
                })))
            } else {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "message": format!("No record found with id: {}", id),
                })))
            }
        },
        Err(err) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": format!("Database error: {}", err),
        })))
    }
}