use std::time::Instant;

fn main() {
    let start = Instant::now();
    let mut array = vec![1, 4, 7, 9, 3, 5];
    let mut final_array: Vec<i32> = Vec::new();
    println!("Unsorted:  {:?}", array);

    while !array.is_empty() {
        let mut min_num = 0;
        for i in 1..array.len() {
            if array[i] < array[min_num] {
                min_num = i;
            }
        }
        final_array.push(array.remove(min_num));
    }

    println!("Sorted:    {:?}", final_array);
    println!("Completed: {:?}", start.elapsed());
}
