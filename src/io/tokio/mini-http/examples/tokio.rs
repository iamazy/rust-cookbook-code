use futures::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::task;
use std::io;
use bytes::BytesMut;
use mini_http::{Response, Request};

async fn process(mut stream: TcpStream) -> io::Result<()> {
    let mut v =vec![0u8; 16 * 1024];
    let mut input = BytesMut::new();
    let mut output = BytesMut::new();

    loop {
        match stream.read(&mut v).await? {
            0 => return Ok(()),
            n => input.extend_from_slice(&v[..n]),
        }

        while let Some(_) = Request::decode(&mut input)?{
            Response::new()
                .header("Content-Type","text/plain")
                .body("Hello World")
                .encode(&mut output);
            stream.write_all(&output).await?;
            output.clear();
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut listener = TcpListener::bind("0.0.0.0:8080").await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        task::spawn(process(stream?));
    }
    Ok(())
}