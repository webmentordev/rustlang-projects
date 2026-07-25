fn main() {
    let arr_1 = vec!["Hi", "my", "name", "is", "Ahmer"];
    let mut temp = Vec::new();
    println!("{:?}", arr_1);

    let start = 0;
    let mut end = arr_1.len() as i32 - 1;

    while start <= end {
        temp.push(arr_1[end as usize]);
        end -= 1;
    }
    println!("{:?}", temp);
}
