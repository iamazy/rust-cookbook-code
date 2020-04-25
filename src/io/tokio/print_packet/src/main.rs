use tokio;
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::{BytesCodec,Decoder};

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
        let (socket,_) = listener.accept().await?;
        tokio::spawn(async move {
            let mut framed = BytesCodec::new().framed(socket);
            while  let Some(message) = framed.next().await {
                match message {
                    Ok(bytes) => println!("bytes: {:?}",bytes),
                    Err(e) => println!("Socket closed with error: {:?}",e),
                }
            }
            println!("Socket received FIN packet and closed connection.");
        });
    }
    
}