use serde::{Deserialize, Deserializer, Serializer};

#[derive(Debug)]
struct Color(u8, u8, u8);

impl serde::Serialize for Color {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        let hex = s.trim_start_matches('#');
        Ok(Color(
            u8::from_str_radix(&hex[0..2], 16).unwrap(),
            u8::from_str_radix(&hex[2..4], 16).unwrap(),
            u8::from_str_radix(&hex[4..6], 16).unwrap(),
        ))
    }
}

fn main() {
    let c = Color(255, 128, 0);
    let json = serde_json::to_string(&c).unwrap();
    println!("{}", json); // "#FF8000"

    let back: Color = serde_json::from_str(&json).unwrap();
    println!("{:?}", back); // Color(255, 128, 0)
}
