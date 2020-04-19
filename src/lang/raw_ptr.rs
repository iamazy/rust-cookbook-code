
fn main() {
    let mut x = 10;
    let ptr_x = &mut x as *mut i32;
    // 20存储在堆上
    let y  =  Box::new(20);
    let ptr_y = &*y as *const i32;
    unsafe { 
        *ptr_x +=*ptr_y;
    }
    println!("{:?}",x);
}