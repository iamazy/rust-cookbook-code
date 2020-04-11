pub fn literals_operators() {
    //指定数字类型
    println!("1 + 2 = {}", 1u32 + 2);
    //1u32 -2 会报错，因为返回值不在u32的范围之内
    println!("1 - 2 = {}", 1i32 - 2);
    println!("true && false is {}", true && false);
    println!("true || false is {}", true || false);
    println!("not true is {}", !true);

    //bitwise operations
    println!("0011 and 0101 is {:04b}", 0b0011u32 & 0b0101);
    println!("0011 or 0101 is {:04b}", 0b0011u32 | 0b0101);
    println!("0011 xor 0101 is {:04b}", 0b0011u32 ^ 0b0101);
    println!("1 << 5 is {}", 1u32 << 5);
    println!("0x80 >> 2 is 0x{:x}", 0x80u32 >> 2);
}
