use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};


fn main() {
    let mut rdr = Cursor::new(vec![2,5,3,0]);
    println!("{:?}", rdr.read_u16::<BigEndian>().unwrap());
}