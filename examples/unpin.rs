use std::pin::Pin;

fn main() {
    let mut string = "this".to_string();
    let mut pinned_string = Pin::new(&mut string);

    std::mem::replace(&mut *pinned_string, "other".to_string());

    println!("{}",string);
}