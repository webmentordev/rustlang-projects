use headless_chrome::{Browser, LaunchOptions};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::task;

#[derive(Deserialize)]
struct UrlConfig {
    _api: String,
    url: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    let args: Vec<String> = env::args().collect();

    let concurrent = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(3);

    let total = args
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50);

    let start = args
        .get(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    let config_data = fs::read_to_string("info.json")
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
    let config: UrlConfig = serde_json::from_str(&config_data)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    let pages_dir = Path::new("pages");
    if !pages_dir.exists() {
        fs::create_dir(pages_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        println!("Created 'pages' directory");
    }

    let start_time = Instant::now();

    println!("========================================");
    println!("Configuration:");
    println!("  Concurrent pages: {}", concurrent);
    println!("  Total pages per URL: {}", total);
    println!("  Starting page: {}", start);
    println!("  URLs found: {}", config.url.len());
    for (key, url) in &config.url {
        println!("    - {}: {}", key, url);
    }
    println!("========================================\n");

    let user_agent = OsString::from(
        "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    );
    let disable_images = OsString::from("--blink-settings=imagesEnabled=false");

    let launch_options = LaunchOptions {
        headless: true,
        window_size: Some((1920, 1080)),
        path: Some("./chrome-linux64/chrome".into()),
        sandbox: false,
        args: vec![&user_agent, &disable_images],
        ..Default::default()
    };

    let browser = Browser::new(launch_options)?;

    let end_page = start + total - 1;

    let mut all_pages: Vec<(String, String, usize)> = Vec::new();
    for (key, base_url) in &config.url {
        for page_num in start..=end_page {
            let url = format!("{}?page={}", base_url, page_num);
            all_pages.push((key.clone(), url, page_num));
        }
    }

    println!("Total pages to scrape: {}\n", all_pages.len());

    for chunk in all_pages.chunks(concurrent) {
        let handles: Vec<_> = chunk
            .iter()
            .map(|(key, url, page_num)| {
                let browser = browser.clone();
                let url = url.clone();
                let key = key.clone();
                let page_num = *page_num;
                task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send>> {
                    let tab = browser.new_tab()?;
                    println!("Navigating to: {} ({})", url, key);
                    tab.navigate_to(&url)?;
                    match tab.wait_for_element_with_custom_timeout(
                        "div.tm-property-premium-listing-card__details-container",
                        Duration::from_secs(30),
                    ) {
                        Ok(_) => println!("✓ Page loaded successfully: {} ({})", url, key),
                        Err(e) => {
                            println!("⚠ Timeout waiting for content on {} ({}): {}", url, key, e);
                        }
                    }

                    let html = tab.get_content()?;
                    let filename = format!("pages/{}_page_{}.html", key, page_num);
                    std::fs::write(&filename, &html)
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

                    println!("✓ Saved rendered HTML to {}", filename);
                    tab.close(true)?;
                    Ok(())
                })
            })
            .collect();

        for handle in handles {
            handle
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)??;
        }
    }

    let elapsed = start_time.elapsed();
    let total_seconds = elapsed.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    println!("\n========================================");
    println!("Total time: {}h {}m {}s", hours, minutes, seconds);
    println!("========================================");

    Ok(())
}
