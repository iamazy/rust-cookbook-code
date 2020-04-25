//! A simple client that opens a TCP stream, writes "hello world\n", and closes
//! the connection.
//!
//! You can test this out by running:
//!
//!     ncat -l 6142

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use std::error::Error;


#[tokio::main]
pub async fn main() -> Result<(),Box<dyn Error>> {

    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    println!("create stream");

    let result = stream.write(b"Hello World\n").await;
    println!("wrote to stream; success={:?}",result.is_ok());

    Ok(())
}