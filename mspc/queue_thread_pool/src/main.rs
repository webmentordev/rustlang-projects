use std::sync::mpsc;
use std::thread;

fn main() {
    let (result_tx, result_rx) = mpsc::channel::<(u32, u64)>();

    let numbers: Vec<u32> = (1..=8).collect();

    for n in numbers {
        let tx = result_tx.clone();
        thread::spawn(move || {
            let result: u64 = (1..=n as u64).sum();
            tx.send((n, result)).unwrap();
        });
    }

    drop(result_tx);

    let mut results: Vec<(u32, u64)> = result_rx.iter().collect();
    results.sort();
    for (n, sum) in results {
        println!("sum(1..{}) = {}", n, sum);
    }
}
