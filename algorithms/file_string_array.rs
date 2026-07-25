use std::fs;
use std::time::Instant;

fn main() {
    let strat = Instant::now();
    let text = fs::read_to_string("content.txt").expect("File not found!");
    let mut result_array = Vec::new();
    let mut word_start = 0;

    for index in 0..text.len() {
        if &text[index..index + 1] == " " {
            result_array.push(&text[word_start..index]);
            word_start = index + 1;
        }
    }

    if word_start < text.len() {
        result_array.push(&text[word_start..]);
    }

    // println!("{:?}", result_array);
    println!("Completed: {:?}", strat.elapsed());
}
