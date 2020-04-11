mod algorithms;
mod compress;
mod concurrency;
mod crypt;
mod examples;
mod exercises;
mod raft;

fn main() {
    main_raft();
}

fn main_raft() {
    raft::single_node::single_node();
}

fn main_func() {
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
    // 耗时
    //compress::decompress_tarball().unwrap();
    compress::compress_dir().unwrap();

    println!("-------------------------------");
    let arr = &[1, 25, -4, 10];
    let max = concurrency::explicit_threads::find_max(arr);
    println!("{:?}", max);

    concurrency::explicit_threads::pass_data_in_two_threads();
    concurrency::explicit_threads::maintain_global_mutable_state().unwrap();
    concurrency::explicit_threads::calculate_sha256_sum_of_iso_concurrently().unwrap();
    // 耗时
    // concurrency::explicit_threads::draw_fractal().unwrap();

    println!("-------------------------------");
    concurrency::parallel_tasks::mutate_arr_element_parallel();
    concurrency::parallel_tasks::test_parallel_predicate();
    concurrency::parallel_tasks::find_item_in_parallel();
    concurrency::parallel_tasks::sort_vec_parallel();
    concurrency::parallel_tasks::map_reduce_parallel();

    println!("-------------------------------");
    crypt::hashing::calculate_sha256_digest_of_file().unwrap();
    crypt::hashing::sign_verify_message_with_hmac_digest().unwrap();
    crypt::encryption::salt_hash_password_with_pbkdf2().unwrap();

    println!("-------------------------------");
    println!("{:?}", exercises::word_count::word_count("hello world1"));

    fn square(x: i32) -> i32 {
        x * x
    }
    println!("{:?}", exercises::accumulate::map(vec![1, 2, 3], square));
    println!("{:?}", exercises::accumulate::map(vec![1, 2, 3], |x| x + x));

    use exercises::accumulate::do_twice;
    let mut x: usize = 1;
    {
        let add_two_to_x = || x += 2;
        do_twice(add_two_to_x);
    }
    println!("x: {}", x);
    println!("{:?}", exercises::acronym::abbreviate("hellO World"));

    println!("------------------------------");
    examples::literals_operators::literals_operators();
}
