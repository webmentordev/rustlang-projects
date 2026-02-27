use std::sync::mpsc;
use std::thread;
use std::time::Duration;

enum AppEvent {
    UserLogin(String),
    DataLoaded(Vec<String>),
    Error(String),
}

fn main() {
    let (tx, rx) = mpsc::channel::<AppEvent>();

    let tx1 = tx.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        tx1.send(AppEvent::UserLogin("alice".to_string())).unwrap();
    });

    let tx2 = tx.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_micros(200));
        tx2.send(AppEvent::DataLoaded(vec!["item1".into(), "item2".into()]))
            .unwrap();
    });

    drop(tx);

    for event in rx {
        match event {
            AppEvent::UserLogin(user) => println!("🟢 User logged in: {}", user),
            AppEvent::DataLoaded(items) => println!("📦 Loaded {} items", items.len()),
            AppEvent::Error(e) => eprintln!("🔴 Error: {}", e),
        }
    }
}
