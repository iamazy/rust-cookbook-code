extern crate rand;

use rand::distributions::{Alphanumeric, Distribution, Normal, Standard, Uniform};
use rand::Rng;

pub fn gen_rand_num() {
    let mut rng = rand::thread_rng();

    let n1: u8 = rng.gen();
    let n2: u16 = rng.gen();

    println!("Random u8:{}", n1);
    println!("Random u16:{}", n2);
    println!("Random u32:{}", rng.gen::<u32>());
    println!("Random i32:{}", rng.gen::<i32>());
    println!("Random float:{}", rng.gen::<f64>());
}

pub fn gen_rand_num_in_range() {
    let mut rng = rand::thread_rng();
    println!("Integer:{}", rng.gen_range(0, 10));
    println!("Float:{}", rng.gen_range(0.0, 10.0));
}

pub fn gen_rand_num_in_range_faster() {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(1..7);

    loop {
        let throw = die.sample(&mut rng);
        println!("Roll the die:{}", throw);
        if throw == 6 {
            break;
        }
    }
}

pub fn gen_rand_num_with_distribution() {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(2.0, 3.0);
    let v = normal.sample(&mut rng);
    println!("{} is from a N(2,9) distribution", v);
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Distribution<Point> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point {
        let (rand_x, rand_y) = rng.gen();
        Point {
            x: rand_x,
            y: rand_y,
        }
    }
}

pub fn gen_rand_num_of_custom_type() {
    let mut rng = rand::thread_rng();

    // 指定返回值类型的两种方式
    let rand_tuple = rng.gen::<(i32, bool, f64)>();
    let rand_point: Point = rng.gen();
    println!("Random tuple: {:?}", rand_tuple);
    println!("Random Point: {:?}", rand_point);
}

pub fn gen_rand_password() {
    let rand_str: String = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(30)
        .collect();

    println!("{:?}", rand_str);
}

pub fn gen_rand_password_from_user_defined() {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const PASSWORD_LENGTH: usize = 30;

    let mut rng = rand::thread_rng();
    let password: String = (0..PASSWORD_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    println!("{:?}", password);
}
