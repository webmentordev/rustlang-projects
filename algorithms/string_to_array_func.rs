fn main() {
    let text = "Hi, my name is Ahmer. Nice to meet you!";
    let txt_array: Vec<&str> = text.split(" ").collect();
    let txt_array_2: Vec<&str> = text.split_whitespace().collect();
    println!("{:?}", txt_array);
    println!("{:?}", txt_array_2);
}

