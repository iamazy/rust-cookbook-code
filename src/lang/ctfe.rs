const fn init_len() -> usize {
    5
}

fn main() {
    let arr = [0; init_len()];
    println!("{:?}", arr);
}
