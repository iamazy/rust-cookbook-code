#[derive(Debug,PartialEq)]
struct People {
    name: &'static str,
    gender: u32,
}

impl People {
    fn new(name: &'static str, gender: u32) -> Self {
        return People { name: name, gender: gender };
    }

    fn name(&self) {
        println!("name: {:?}", self.name);
    }

    fn set_name(&mut self, name: &'static str) {
        self.name = name;
    }

    fn gender(&self) {
        let gender = if (self.gender == 1) {
            "boy"
        } else {
            "girl"
        };
        println!("gender: {:?}", gender);
    }
}

fn main() {
    let alex = People::new("Alex",1);
    alex.name();
    alex.gender();
    let mut alice = People::new("Alice",0);
    alice.name();
    alice.gender();

    alice.set_name("Rose");
    alice.name();

    println!("{:?}",alex);
}