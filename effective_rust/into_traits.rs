// IMPORTANT: This code will not compile with both trait implementation applied at once
// Use TryFrom trait for flexibility and it provides an equivalent TryInto implementation
// The struct implementation here is juust an example, you can use Display Trait for this

// Into has two traits, TryInto & Into
// Use which suits the need.
// TryInto, try_into(), returns Result which indicates Success or Fail
// Into, into(), always succeed

struct Person {
    name: String,
    age: i64,
}

impl TryInto<String> for Person {
    type Error = String;
    fn try_into(self) -> Result<String, Self::Error> {
        if self.age < 0 {
            Err("Why do you wanna unborn?".to_string())
        } else {
            Ok(format!("Hi, {} years old {}", self.age, self.name))
        }
    }
}

impl Into<String> for Person {
    fn into(self) -> String {
        format!("Hi, {} years old {}", self.age, self.name)
    }
}

fn main() {
    // The OK & Error types must be known
    let person: Result<String, String> = Person {
        name: "Ahmer".to_string(),
        age: 76,
    }
    .try_into();
    match person {
        Ok(msg) => println!("{}", msg),
        Err(e) => println!("{}", e),
    }

    let student: String = Person {
        name: "Kaleem".to_string(),
        age: 8,
    }
    .into();
    println!("{student}");
}
