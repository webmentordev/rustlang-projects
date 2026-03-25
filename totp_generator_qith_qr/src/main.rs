use base64::decode;
use std::fs::File;
use std::io::Write;
use totp_rs::{Algorithm, Secret, TOTP};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded("KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ".to_string())
            .to_bytes()
            .unwrap(),
        Some("Github".to_string()),
        "ahmerdev@github.com".to_string(),
    )?;

    let qr_base64 = totp.get_qr_base64()?;

    let image_bytes = decode(qr_base64)?;

    let mut file = File::create("qrcode.png")?;
    file.write_all(&image_bytes)?;

    println!("QR code saved as qrcode.png");

    Ok(())
}
