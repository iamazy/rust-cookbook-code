fn main() {
    let mut a = vec![1, 2, 3];
    let b = &mut a;
    println!("{:p}", b);
    b.push(4);
    println!("{:?}", a);

    let e = &42;
    assert_eq!(*e, 42);
}
