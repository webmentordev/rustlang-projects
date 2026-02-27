use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

fn main() {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("logs.log")
        .unwrap();
    let (tx, rx) = mpsc::channel::<String>();

    for id in 0..4 {
        let tx = tx.clone();
        thread::spawn(move || {
            for i in 0..3 {
                let msg = format!("[Worker-{}] Task {} completed", id, i);
                tx.send(msg).unwrap();
            }
        });
    }
    drop(tx);

    for log in rx {
        writeln!(
            file,
            "[{}] {}",
            Local::now().format("%Y-%m-%d %I:%M %p"),
            log
        )
        .unwrap();
        println!("{}", log);
    }
}
