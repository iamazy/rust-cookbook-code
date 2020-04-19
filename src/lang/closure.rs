
fn main() {
    let out = 42;

    fn add(i: i32, j: i32) -> i32 { i + j}
    let closure_annotated = |i: i32, j: i32| -> i32 { i + j + out };
    let closure_inferred = |i,j| i+j+out;
    let i = 1;
    let j = 2;
    println!("{:?}",add(i, j));
    println!("{:?}",closure_annotated(i, j));
    println!("{:?}",closure_inferred(i, j));
    println!("{:?}",math(||i+j));
    println!("{:?}",math(||i*j));

    let result = two_times_impl();
    println!("{:?}",result(2));
}

fn math<F: Fn() -> i32>(op: F) -> i32 { 
    op()
}

fn two_times_impl() -> impl Fn(i32) -> i32 { 
    let i =2;
    move |j| j * i
}



