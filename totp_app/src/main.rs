use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};

fn main() {
    let secret = b"JBSWY3DPEHPK3PXP"; // This is a Secret Key used as an exmaple.

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.to_vec()).unwrap();

    let token = totp.generate_current().unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let time_left = 30 - (now % 30);

    println!("Current TOTP: {}", token);
    println!("Expires in:   {}s", time_left);
}
