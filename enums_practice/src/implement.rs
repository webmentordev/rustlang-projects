enum Animal {
    Dog,
    Cat,
    Bird,
    Rat(String),
}

impl Animal {
    fn is_loud(&self) -> bool {
        matches!(self, Animal::Cat | Animal::Dog)
    }

    fn sound(&self) {
        match self {
            Animal::Dog => println!("Woof"),
            Animal::Bird => println!("Tweet"),
            Animal::Cat => println!("Meaw"),
            Animal::Rat(s) => println!("{}", s),
        }
    }
}

fn main() {
    let animal = Animal::Rat("Squeek".to_string());
    if animal.is_loud() {
        println!("Animal is Loud");
    } else {
        println!("Animal is silent!");
    }
    animal.sound();
}
