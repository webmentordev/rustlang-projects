use std::sync::mpsc;
use std::thread;

struct ScrappedData {
    url: String,
    word_count: usize,
}

fn scrape(url: &str) -> ScrappedData {
    ScrappedData {
        url: url.to_string(),
        word_count: url.len() * 10,
    }
}

fn main() {
    let urls = vec![
        "https://example.com",
        "https://rust-lang.org",
        "https://docs.rs",
    ];

    let (tx, rx) = mpsc::channel::<ScrappedData>();

    for url in urls {
        let tx = tx.clone();
        thread::spawn(move || {
            let data = scrape(url);
            tx.send(data).unwrap();
        });
    }
    drop(tx);

    let mut total_words = 0;
    for result in rx {
        println!("URL: {} | Words: {}", result.url, result.word_count);
        total_words += result.word_count;
    }
    println!("Total words scraped: {}", total_words);
}
