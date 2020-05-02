use std::env;
use std::error::Error;
use std::io::{stdin,Read};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

/// type in terminal and end with `EOF` (Ctrl+D)
fn get_stdin_data() -> Result<Vec<u8>,Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
                    .nth(1)
                    .unwrap_or_else(||"127.0.0.1:8080".into())
                    .parse()?;

    // we use port 0 to let the operating system allocate an available port for us
    let local_addr: SocketAddr = if remote_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    }.parse()?;
    let mut socket = UdpSocket::bind(local_addr).await?;
    const MAX_DATAGRAM_SIZE: usize = 65_507;
    socket.connect(&remote_addr).await?;
    let data = get_stdin_data()?;
    socket.send(&data).await?;
    let mut data = vec![0u8;MAX_DATAGRAM_SIZE];
    let len = socket.recv(&mut data).await?;
    println!("Received {} bytes:\n{}",len,String::from_utf8_lossy(&data[..len]));
    Ok(())
}