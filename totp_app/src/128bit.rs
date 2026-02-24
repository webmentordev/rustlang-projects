use base32::Alphabet::RFC4648;
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret = "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"; //  128 bits string key.
    let secret_clean = secret.trim().trim_end_matches('=');
    let secret_bytes = base32::decode(RFC4648 { padding: false }, secret_clean)
        .ok_or("Failed to decode Base32 secret")?;
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)
        .map_err(|e| format!("Failed to create TOTP: {}", e))?;
    let token_str = totp
        .generate_current()
        .map_err(|e| format!("Failed to generate token: {}", e))?;
    let token: u32 = token_str
        .parse()
        .map_err(|e| format!("Failed to parse token as u32: {}", e))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let time_left = 30 - (now % 30);

    println!("Token: {}, Time until expire: {}s", token, time_left);
    Ok(())
}
