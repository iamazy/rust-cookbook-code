pub fn sort_int_vec() {
    let mut vec = vec![1, 5, 10, 2, 15];
    vec.sort();
    println!("{:?}", vec);
}

pub fn sort_float_vec() {
    let mut vec = vec![1.1, 1.15, 5.5, 1.2, 2.0];
    vec.sort_by(|x, y| x.partial_cmp(y).unwrap());
    println!("{:?}", vec);
}
