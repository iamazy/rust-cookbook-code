use std::error::Error;
use tokio::sync::{mpsc, Mutex};
use tokio::net::{TcpStream, TcpListener};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::stream::{StreamExt, Stream};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use futures::{SinkExt};
use std::sync::Arc;
use std::io;
use std::pin::Pin;
use futures::task::{Context, Poll};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = Arc::new(Mutex::new(Shared::new()));

    let addr = env::args()
        .nth(1).unwrap_or_else(||"127.0.0.1:6142".to_string());

    let mut listener = TcpListener::bind(&addr).await?;
    println!("server running on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = process(state, stream, addr).await {
                println!("an error occurred; error = {:?}", e);
            }
        });
    }
}


/// Shorthand for the transmit half of the message channel
type Tx = mpsc::UnboundedSender<String>;
/// Shorthand for the receive half of the message channel
type Rx = mpsc::UnboundedReceiver<String>;

struct Shared {
    peers: HashMap<SocketAddr, Tx>
}

struct Peer {
    lines: Framed<TcpStream, LinesCodec>,
    rx: Rx
}

impl Shared {
    fn new() -> Self {
        Shared {
            peers: HashMap::new()
        }
    }

    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}

impl Peer {
    async fn new(
        state: Arc<Mutex<Shared>>,
        lines: Framed<TcpStream, LinesCodec>,
    ) -> io::Result<Peer> {
        let addr = lines.get_ref().peer_addr()?;
        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();
        // Add an entry for this `Peer` in the shared state map.
        state.lock().await.peers.insert(addr, tx);
        Ok(Peer {
            lines,
            rx
        })
    }
}

#[derive(Debug)]
enum Message {
    /// A message that should be broadcasted to others
    Broadcast(String),
    /// A message that should be received by client
    Received(String)
}

impl Stream for Peer {
    type Item = Result<Message, LinesCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // First poll the `UnboundedReceiver`
        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            return Poll::Ready(Some(Ok(Message::Received(v))));
        }
        // Secondly poll the `Framed` stream
        let result: Option<_> = futures::ready!(Pin::new(&mut self.lines).poll_next(cx));
        Poll::Ready(match result {
            Some(Ok(message)) => Some(Ok(Message::Broadcast(message))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        })
    }
}

async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    lines.send("Please enter your username:").await?;

    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            println!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(())
        }
    };

    let mut peer = Peer::new(state.clone(), lines).await?;
    // A client has connected, let's everyone know
    {
        let mut state = state.lock().await;
        let msg = format!("{} has joined the chat",username);
        println!("{}", msg);
        state.broadcast(addr, &msg).await;
    }
    // Process incoming messages until our stream is exhausted by a disconnect
    while let Some(result) = peer.next().await {
        match result {
            Ok(Message::Broadcast(msg)) => {
                let mut state = state.lock().await;
                let msg = format!("{}: {}",username, msg);

                state.broadcast(addr, &msg).await;
            }

            // A message was receive from a peer. Send it to the current user
            Ok(Message::Received(msg)) => {
                peer.lines.send(&msg).await?;
            }
            Err(e) => {
                println!("an error occurred while processing messages for {}; error = {:?}", username, e);
            }
        }
    }

    {

        let mut state = state.lock().await;
        state.peers.remove(&addr);

        let msg = format!("{} has left the chat", username);
        println!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    Ok(())
}