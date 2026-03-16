#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum Animal {
    Dog,
    Cat,
    Bird,
    Rat(String),
}

#[allow(unused_variables)]
fn main() {
    println!("{:?}", Animal::Dog);
    let animal = Animal::Rat("Squeek".to_string());
    println!("{:?}", animal);
}
