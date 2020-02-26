use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

pub fn mutate_arr_element_parallel() {
    let mut arr = [0, 7, 9, 11];
    arr.par_iter_mut().for_each(|p| *p -= 1);
    println!("{:?}", arr);
}

pub fn test_parallel_predicate() {
    let mut vec = vec![2, 4, 6, 8];
    println!("{:?}", vec.par_iter().any(|n| (*n % 2) != 0));
    println!("{:?}", vec.par_iter().all(|n| (*n % 2) == 0));
    println!("{:?}", vec.par_iter().any(|n| *n > 8));
    println!("{:?}", vec.par_iter().all(|n| *n <= 8));

    vec.push(9);
    println!("{:?}", vec.par_iter().any(|n| (*n % 2) != 0));
    println!("{:?}", vec.par_iter().all(|n| (*n % 2) == 0));
    println!("{:?}", vec.par_iter().any(|n| *n > 8));
    println!("{:?}", vec.par_iter().all(|n| *n <= 8));
}

pub fn find_item_in_parallel() {
    let vec = vec![6, 2, 1, 9, 3, 8, 11];

    let f1 = vec.par_iter().find_any(|&&n| n == 9);
    let f2 = vec.par_iter().find_any(|&&n| n % 2 == 0 && n > 6);
    let f3 = vec.par_iter().find_any(|&&n| n > 8);

    println!("{:?}", f1.unwrap());
    println!("{:?}", f2);
    println!("{:?}", f3);
}

pub fn sort_vec_parallel() {
    let mut vec = vec![String::new(); 10];
    println!("unsort:{:?}", vec);
    vec.par_iter_mut().for_each(|p| {
        let mut rng = thread_rng();
        *p = (0..5).map(|_| rng.sample(Alphanumeric)).collect();
    });
    vec.par_sort_unstable();
    println!("sorted:{:?}", vec);
}

struct Person {
    age: u32,
}

pub fn map_reduce_parallel() {
    let v: Vec<Person> = vec![Person { age: 10 }, Person { age: 40 }];

    let num_over_30 = v
        .par_iter()
        // .filter(|&x| x.age > 30) 也行
        .filter(|x| x.age > 30)
        .count() as f32;
    let sum_over_30 = v
        .par_iter()
        .map(|x| x.age)
        // .filter(|&x|x>30)也行
        .filter(|age| *age > 30)
        .reduce(
            || 0,
            |x, y| {
                println!("{}:{}", x, y);
                (x + y)
            },
        );
    println!("{:?}", sum_over_30);

    let alt_sum_30: u32 = v.par_iter().map(|x| x.age).filter(|x| *x > 30).sum();

    let avg_over_30 = sum_over_30 as f32 / num_over_30;
    let alt_avg_sum_30 = alt_sum_30 as f32 / num_over_30;

    println!("{:?}", (avg_over_30 - alt_avg_sum_30).abs());
    println!("{:?}", std::f32::EPSILON);
    println!("{:?}", avg_over_30);
    println!("{:?}", alt_avg_sum_30);
}
