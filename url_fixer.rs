use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("program <url-to-fix>");
        return;
    }

    let fixed = args[1].replace("\\", "");
    println!("URL: {}", fixed);
}
