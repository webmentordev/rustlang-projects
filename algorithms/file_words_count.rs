use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

fn main() {
    // Count words in a file, performance
    let start = Instant::now();
    let file = File::open("content.txt").expect("File not found!");
    let content = BufReader::new(file);
    let mut words_count = 0;
    let lines = content.lines().flatten();

    for line in lines {
        let mut word_start = 0;
        for index in 0..line.len() {
            if &line[index..index + 1] == " " {
                words_count += 1;
                word_start = index + 1;
            }
        }
        if word_start < line.len() {
            words_count += 1;
        }
    }

    println!("Total words: {:?}", words_count);
    println!("Completed: {:?}", start.elapsed());
}
