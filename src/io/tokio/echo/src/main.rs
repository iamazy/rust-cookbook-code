use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {

    let addr = env::args()
                .nth(1)
                .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let mut listener = TcpListener::bind(&addr).await?;

    println!("Listening on: {}",addr);

    loop {
        let (mut socket,_) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0;1024];

            loop {
                let n = socket.read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                socket.write_all(&buf[0..n])
                        .await
                        .expect("failed to write data to socket");
            }
        });
    }
    
}