fn main() {
    let arr_1 = vec!["Hi", "my", "name", "is", "Ahmer"];
    let length = arr_1.len() - 1;
    let mut temp = String::new();
    println!("{:?}", arr_1);
    for (idx, txt) in arr_1.into_iter().enumerate() {
        temp.push_str(txt);
        if idx < length {
            temp.push_str(" ");
        }
    }
    println!("{}", temp);
}
