use std::fmt;

enum Animal {
    Dog,
    Cat,
    Bird,
    Rat(String),
}

impl fmt::Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Animal::Bird => write!(f, "bird tweets"),
            Animal::Cat => write!(f, "cat meaws"),
            Animal::Dog => write!(f, "dog barks"),
            Animal::Rat(s) => write!(f, "rat {}", s),
        }
    }
}

fn main() {
    let animal = Animal::Dog;
    println!("a {}", animal);

    let animal = Animal::Cat;
    println!("a {}", animal);

    let animal = Animal::Bird;
    println!("a {}", animal);

    let animal = Animal::Rat("squeels".to_string());
    println!("a {}", animal);
}
