fn main() {
    // Bubble Sort
    let mut arr = vec![1, 5, 10, 4, 8, 3, 9];
    let length = arr.len();
    println!("Unsorted: {:?}", arr);

    for i in 0..length {
        let mut swapped = false;
        for j in 0..length - i - 1 {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }

    println!("Sorted: {:?}", arr);
}
