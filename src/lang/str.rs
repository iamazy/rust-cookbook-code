
fn main() {
    let truth: &'static str = "hello rust";
    let ptr = truth.as_ptr();
    let len = truth.len();
    println!("{:}",len); 

    let s = unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        std::str::from_utf8(slice).unwrap()
    };

    println!("{:?}",s);
}