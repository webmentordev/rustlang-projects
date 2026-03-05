use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Records {
    collection: Option<HashMap<String, u16>>,
}

fn main() {
    let mut collect = Records {
        collection: Some(HashMap::new()),
    };

    if let Some(map) = collect.collection.as_mut() {
        map.insert("Yo".to_string(), 1231);
        map.insert("Ao".to_string(), 1232);
        map.insert("Bo".to_string(), 1233);
        map.insert("Co".to_string(), 1234);
        map.insert("Do".to_string(), 1235);
        map.insert("Eo".to_string(), 1236);
        map.insert("Fo".to_string(), 1237);
        map.insert("Go".to_string(), 1238);
    };

    let to_json = serde_json::to_string(&collect).unwrap();
    println!("{}", to_json);

    let to_struct: Records = serde_json::from_str(&to_json).unwrap();
    println!("{:?}", to_struct);
}
