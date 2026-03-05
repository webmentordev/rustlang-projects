use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 88, y: 22 };
    let serialized = serde_json::to_string(&point).unwrap();
    println!("{}", serialized);

    let deserialied: Point = serde_json::from_str(&serialized).unwrap();
    println!("{:?}", deserialied);
}
