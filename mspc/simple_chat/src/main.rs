use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

type ChatBus = Arc<Mutex<HashMap<String, Sender<String>>>>;

fn main() {
    let bus: ChatBus = Arc::new(Mutex::new(HashMap::new()));

    let (alice_tx, alince_rx) = mpsc::channel::<String>();
    bus.lock().unwrap().insert("Alice".into(), alice_tx);

    let (bob_tx, bob_rx) = mpsc::channel::<String>();
    bus.lock().unwrap().insert("Bob".into(), bob_tx);

    let bus_clone = Arc::clone(&bus);
    thread::spawn(move || {
        let bus = bus_clone.lock().unwrap();
        if let Some(sender) = bus.get("Bob") {
            sender.send("Hey Bob, It's Alice".into()).unwrap();
        }
    });

    let msg = bob_rx.recv().unwrap();
    println!("[Bob received]: {}", msg);

    drop(alince_rx);
}
