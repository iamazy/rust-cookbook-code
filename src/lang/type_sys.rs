
fn reset(mut arr: [u32;2]) {
    arr[1] = 1;
    arr[0] = 0;
    println!("{:?}",arr);
}

fn reset1(arr: &mut [u32]){
    arr[0] = 0;
    arr[1] = 1;
    println!("{:?}",arr);
}

fn main() {
    let mut arr = [3,2];
    println!("{:?}",arr);
    {
        let mut_arr: &mut [u32] = &mut arr;
        reset1(mut_arr);
    }
    println!("{:?}",arr);
}