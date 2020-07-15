use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use futures::future::try_join;
use futures::FutureExt;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr = env::args().nth(1)
        .unwrap_or_else(||"127.0.0.1:8081".to_string());
    let server_addr = env::args().nth(2)
        .unwrap_or_else(||"127.0.0.1:8080".to_string());

    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    let mut listener = TcpListener::bind(listen_addr).await?;
    while let Ok((inbound, _)) = listener.accept().await {
        let transfer = transfer(inbound, server_addr.clone()).map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer; error: {}", e);
            }
        });
        tokio::spawn(transfer);
    }
    Ok(())
}

async fn transfer(mut inbound: TcpStream, proxy_addr: String) -> Result<(), Box<dyn Error>> {
    let mut outbound = TcpStream::connect(proxy_addr).await?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);


    try_join(client_to_server, server_to_client).await?;

    Ok(())
}