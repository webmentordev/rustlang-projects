#[derive(Debug)]
enum Animal {
    Dog,
    Cat,
    Bird,
    Rat(String),
}

enum Cage<T> {
    Occupied(T),
    Empty,
}

fn main() {
    let cage: Cage<Animal> = Cage::Occupied(Animal::Rat("Squeek".to_string()));
    match cage {
        Cage::Occupied(_) => println!("Cage is occupied!"),
        Cage::Empty => println!("Nothing here!"),
    }

    // With name
    let cage: Cage<Animal> = Cage::Occupied(Animal::Dog);
    match cage {
        Cage::Occupied(a) => println!("Cage is occupied by the {:?}", a),
        Cage::Empty => println!("Nothing here!"),
    }
}
