use std::fmt::*;
struct Duck;
struct Pig;

trait Fly {
    fn fly(&self) -> bool;
}

impl Fly for Duck {
    fn fly(&self) -> bool {
        return true;
    }
}

impl Fly for Pig {
    fn fly(&self) -> bool {
        false
    }
}

fn fly_static<T: Fly>(s: T) -> bool {
    s.fly()
}

fn fly_dyn(s: &dyn Fly) -> bool {
    s.fly()
}

struct Point {
    x: i32,
    y: i32,
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}
fn main() {
    let origin = Point { x: 0, y: 0 };
    println!("The origin is : {:?}", origin);
}
