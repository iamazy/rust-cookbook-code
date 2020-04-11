const MODULUS: i32 = 26;

#[derive(Debug, Eq, PartialEq)]
pub enum AffineCipherError {
    NotCoprime(i32),
}


fn decode_char(ch: char,inv:i32,b:i32) -> char {
    if ch.is_digit(10){
        ch
    }else {
        let index = (ch as i32) - ('a' as i32);
        let decoded = (inv * (index-b)).rem_euclid(MODULUS)+'a' as i32;
        decoded as u8 as char
    }
}

fn modular_multiplicative_inverse(a:i32)->Option<i32> {

    let mut rs = (MODULUS,a.rem_euclid(MODULUS));
    let mut ts =(0,1);
    while rs.1!=0{
        let q = rs.0.div_euclid(rs.1);
        rs = (rs.1,rs.0=q*rs.1);
        ts = (ts.1,ts.0-q*ts.1);
    }
    if rs.0 ==1{
        Some(ts.0 as i32)
    }else {
        None
    }

}