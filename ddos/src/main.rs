use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::time::sleep;

struct ConnectionData {
    count: u32,
    first_seen: Instant,
    last_seen: Instant,
}

struct ServerStats {
    total_connections: u32,
    unique_ips: u32,
    connections_per_second: f32,
    baseline: Option<f32>,
}

#[derive(Debug, serde::Deserialize)]
struct GeoIpResponse {
    status: String,
    country: Option<String>,
    city: Option<String>,
}

#[tokio::main]
async fn main() {
    println!("DDoS Detection Tool Starting...");
    println!("Monitoring network connections...");

    let mut ip_connections: HashMap<String, ConnectionData> = HashMap::new();
    let mut last_cleanup = Instant::now();
    let mut last_stats_time = Instant::now();
    let mut server_stats = ServerStats {
        total_connections: 0,
        unique_ips: 0,
        connections_per_second: 0.0,
        baseline: None,
    };

    let threshold_multiplier = 3.0;
    let learning_period = 60;
    let cleanup_interval = 30;

    println!("Establishing baseline for {} seconds...", learning_period);

    loop {
        let output = Command::new("sh")
            .arg("-c")
            .arg("netstat -tn | grep ESTABLISHED")
            .output();

        match output {
            Ok(output) => {
                let current_time = Instant::now();
                let connections = String::from_utf8_lossy(&output.stdout);
                let mut current_connections = 0;

                for line in connections.lines() {
                    if let Some(remote_addr) = extract_remote_ip(line) {
                        if !is_local_ip(&remote_addr) {
                            current_connections += 1;
                            server_stats.total_connections += 1;

                            match ip_connections.get_mut(&remote_addr) {
                                Some(data) => {
                                    data.count += 1;
                                    data.last_seen = current_time;
                                }
                                None => {
                                    ip_connections.insert(
                                        remote_addr.clone(),
                                        ConnectionData {
                                            count: 1,
                                            first_seen: current_time,
                                            last_seen: current_time,
                                        },
                                    );
                                    let ip_clone = remote_addr.clone();
                                    tokio::spawn(async move {
                                        sleep(Duration::from_secs(5)).await;
                                        if let Some((city, country)) = fetch_geoip(&ip_clone).await
                                        {
                                            log_ip_to_file(&ip_clone, &city, &country);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }

                let elapsed = current_time.duration_since(last_stats_time).as_secs_f32();
                if elapsed >= 1.0 {
                    server_stats.unique_ips = ip_connections.len() as u32;
                    server_stats.connections_per_second = current_connections as f32 / elapsed;
                    if server_stats.baseline.is_none()
                        && current_time.duration_since(last_stats_time).as_secs() >= learning_period
                    {
                        server_stats.baseline = Some(server_stats.connections_per_second);
                        println!(
                            "Baseline established: {:.2} connections per second",
                            server_stats.baseline.unwrap()
                        );
                    }

                    display_stats(&server_stats);
                    if let Some(baseline) = server_stats.baseline {
                        if server_stats.connections_per_second > baseline * threshold_multiplier {
                            alert_potential_attack(&server_stats, &ip_connections);
                        }
                    }

                    last_stats_time = current_time;
                }

                if current_time.duration_since(last_cleanup).as_secs() >= cleanup_interval {
                    cleanup_old_connections(&mut ip_connections, 300);
                    last_cleanup = current_time;
                }
            }
            Err(e) => {
                eprintln!("Error running netstat: {}", e);
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}

fn extract_remote_ip(netstat_line: &str) -> Option<String> {
    let parts: Vec<&str> = netstat_line.split_whitespace().collect();
    if parts.len() >= 4 {
        if let Some(remote_addr) = parts.get(4) {
            return Some(remote_addr.split(':').next().unwrap_or("").to_string());
        }
    }
    None
}

fn is_local_ip(ip: &str) -> bool {
    ip.starts_with("127.")
        || ip.starts_with("192.168.")
        || ip.starts_with("10.")
        || ip.starts_with("172.")
        || ip == "::1"
        || ip.starts_with("fe80:")
        || ip.starts_with("fc00:")
        || ip.starts_with("fd00:")
}

async fn fetch_geoip(ip: &str) -> Option<(String, String)> {
    let url = format!("http://ip-api.com/json/{}", ip);

    match reqwest::get(&url).await {
        Ok(response) => match response.text().await {
            Ok(body) => match serde_json::from_str::<GeoIpResponse>(&body) {
                Ok(geo_data) => {
                    if geo_data.status == "success" {
                        match (geo_data.city, geo_data.country) {
                            (Some(city), Some(country)) => Some((city, country)),
                            _ => {
                                eprintln!("Incomplete GeoIP data for {}", ip);
                                None
                            }
                        }
                    } else {
                        eprintln!("GeoIP API error for {}: {}", ip, geo_data.status);
                        None
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse GeoIP response for {}: {}", ip, e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Failed to read response for {}: {}", ip, e);
                None
            }
        },
        Err(e) => {
            eprintln!("GeoIP lookup failed for {}: {}", ip, e);
            None
        }
    }
}

fn log_ip_to_file(ip: &str, city: &str, country: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("remote_ips.txt")
    {
        let _ = writeln!(file, "{}: {}, {}", ip, city, country);
    }
}

fn display_stats(stats: &ServerStats) {
    print!(
        "\r\x1B[KConnections: {}/s | Unique IPs: {} | Total: {} ",
        stats.connections_per_second, stats.unique_ips, stats.total_connections
    );

    io::stdout().flush().unwrap();
}

fn cleanup_old_connections(connections: &mut HashMap<String, ConnectionData>, seconds: u64) {
    let current_time = Instant::now();
    connections.retain(|_, data| current_time.duration_since(data.last_seen).as_secs() < seconds);
}

fn alert_potential_attack(stats: &ServerStats, connections: &HashMap<String, ConnectionData>) {
    println!("\n\x1B[31m⚠️  POTENTIAL ATTACK DETECTED ⚠️\x1B[0m");
    println!(
        "Current connection rate: {:.2}/s",
        stats.connections_per_second
    );
    println!("Baseline: {:.2}/s", stats.baseline.unwrap_or(0.0));
    println!(
        "Excess: {:.2}x normal traffic",
        stats.connections_per_second / stats.baseline.unwrap_or(1.0)
    );

    let mut connections_vec: Vec<(&String, &ConnectionData)> = connections.iter().collect();
    connections_vec.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    println!("\nTop potential attackers:");
    for (i, (ip, data)) in connections_vec.iter().take(5).enumerate() {
        println!("{}. IP: {} - {} connections", i + 1, ip, data.count);
    }
    println!("\nContinuing monitoring...");
}
