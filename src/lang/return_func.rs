

fn main() {
    println!("{:?}",true_marker()());
}


fn is_true() -> bool {
    true
}

fn true_marker() -> fn() -> bool {
    is_true
}