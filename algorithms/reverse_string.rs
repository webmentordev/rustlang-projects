fn main() {
    let text = "Hi, my name is Ahmer!";
    let length = text.len();
    let chars: Vec<char> = text.chars().collect();
    let mut reversed = String::new();

    for num in (0..length).rev() {
        reversed.push(chars[num]);
    }
    println!("{}", reversed);
}
