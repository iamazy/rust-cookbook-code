
fn main() {
    let v = "Hello World";

    let v = "Hello Rust"; 
    {
        let v = "Hello World";
    }
    println!("{:?}",v);
}