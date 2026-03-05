use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    age: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    profession: Option<String>,
    created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

// Default values
fn default_role() -> String {
    "guest".to_string()
}
fn default_active() -> bool {
    true
}
#[derive(Serialize, Deserialize, Debug)]
struct Account {
    name: String,
    #[serde(default = "default_role")]
    role: String,
    #[serde(default = "default_active")]
    active: bool,
}

fn main() {
    let user = User {
        name: String::from("Ahmer"),
        age: 27,
        profession: None,
        created_at: Some(String::from("5 Feb, 2026 12:25PM")),
    };

    let to_json = serde_json::to_string(&user).unwrap();
    println!("{}", to_json);

    let shape = Shape::Circle { radius: 5.0 };
    let to_json = serde_json::to_string(&shape).unwrap();
    println!("{}", to_json);

    let json = r#"{"type": "Rectangle", "width":3.0,"height":4.0}"#;
    let from_shape: Shape = serde_json::from_str(&json).unwrap();
    println!("{:?}", from_shape);

    // Default values
    let json = r#"{"name":"Carol"}"#;
    let a: Account = serde_json::from_str(json).unwrap();
    println!("{:?}", a);
}
