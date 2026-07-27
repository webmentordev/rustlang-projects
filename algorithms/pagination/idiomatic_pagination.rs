fn main() {
    let mut result: Vec<String> = Vec::new();
    let pages = 20;
    let window = 6;
    let current = 16;

    if pages < window {
        let result: Vec<String> = (1..=pages).map(|p| p.to_string()).collect();
        println!("{:?}", result);
        return;
    }

    let buffer = 2;
    let start_window = (current - buffer).max(1);
    let end_window = (current + buffer).min(pages);

    result.push("1".to_string());
    result.push("2".to_string());
    if start_window > 3 {
        result.push("...".to_string());
    }

    for page in start_window.max(3)..=end_window {
        result.push(page.to_string());
    }

    if end_window < pages - 1 {
        result.push("...".to_string());
    }

    if end_window < pages {
        result.push(pages.to_string());
    }

    println!("{:?}", result);
}
