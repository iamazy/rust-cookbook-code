mod algorithms;

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
}
