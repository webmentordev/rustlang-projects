#[cfg(target_os = "windows")]
fn get_mac_address() -> Result<String, String> {
    use std::process::Command;
    let output = Command::new("ipconfig")
        .arg("/all")
        .output()
        .map_err(|e| format!("Failed to execute ipconfig: {}", e))?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.contains("Physical Address") {
            if let Some(mac) = line.split(':').nth(1) {
                return Ok(mac.trim().to_string());
            }
        }
    }
    Err("MAC address not found".to_string())
}

#[cfg(target_os = "linux")]
fn get_mac_address() -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    let net_dir = "/sys/class/net";
    let entries =
        fs::read_dir(net_dir).map_err(|e| format!("Failed to read /sys/class/net: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let iface_name = entry.file_name();
        let iface_str = iface_name.to_string_lossy();
        if iface_str == "lo" {
            continue;
        }
        let mac_path = format!("{}/{}/address", net_dir, iface_str);
        if Path::new(&mac_path).exists() {
            let mac_bytes =
                fs::read(&mac_path).map_err(|e| format!("Failed to read MAC address: {}", e))?;

            let mac_str = String::from_utf8(mac_bytes)
                .map_err(|e| format!("Failed to parse MAC address: {}", e))?;

            return Ok(mac_str.trim().to_string());
        }
    }
    Err("No network interface found".to_string())
}

#[cfg(target_os = "macos")]
fn get_mac_address() -> Result<String, String> {
    use std::process::Command;
    let output = Command::new("ifconfig")
        .output()
        .map_err(|e| format!("Failed to execute ifconfig: {}", e))?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.contains("ether") {
            if let Some(mac) = line.split_whitespace().nth(1) {
                return Ok(mac.to_string());
            }
        }
    }
    Err("MAC address not found".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn get_mac_address() -> Result<String, String> {
    Err("MAC address retrieval not supported on this platform".to_string())
}

fn main() {
    use std::fs::File;
    use std::io::Write;

    match get_mac_address() {
        Ok(mac) => {
            println!("MAC Address: {}", mac);

            let platform = if cfg!(target_os = "windows") {
                "Windows"
            } else if cfg!(target_os = "linux") {
                "Linux"
            } else if cfg!(target_os = "macos") {
                "macOS"
            } else {
                "Unknown"
            };

            let content = format!("Platform: {}\nMAC Address: {}\n", platform, mac);

            match File::create("mac_address.txt") {
                Ok(mut file) => match file.write_all(content.as_bytes()) {
                    Ok(_) => println!("MAC address saved to mac_address.txt"),
                    Err(e) => eprintln!("Failed to write to file: {}", e),
                },
                Err(e) => eprintln!("Failed to create file: {}", e),
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
