mod algorithms;
mod compress;
mod concurrency;

fn main() {
    algorithms::gen_rand::gen_rand_num();
    algorithms::gen_rand::gen_rand_num_in_range();
    algorithms::gen_rand::gen_rand_num_in_range_faster();
    algorithms::gen_rand::gen_rand_num_with_distribution();
    algorithms::gen_rand::gen_rand_num_of_custom_type();
    algorithms::gen_rand::gen_rand_password();
    algorithms::gen_rand::gen_rand_password_from_user_defined();

    println!("-------------------------------");
    algorithms::sort_vec::sort_int_vec();
    algorithms::sort_vec::sort_float_vec();

    println!("-------------------------------");
    //compress::decompress_tarball().unwrap();
    compress::compress_dir().unwrap();

    println!("-------------------------------");
    let arr = &[1, 25, -4, 10];
    let max = concurrency::explicit_threads::find_max(arr);
    println!("{:?}", max);

    concurrency::explicit_threads::pass_data_in_two_threads();
    concurrency::explicit_threads::maintain_global_mutable_state();
}
