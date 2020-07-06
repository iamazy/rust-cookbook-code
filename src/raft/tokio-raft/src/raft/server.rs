use std::time::Duration;
use crate::raft::node::Node;
use std::collections::HashMap;
use crate::raft::message::{Message, Request, Response, Address};
use crate::raft::log::Log;
use crate::error::Result;
use crate::raft::state::State;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;
use futures::{sink::SinkExt, FutureExt};
use ::log::{debug, error};


/// The duration of a Raft tick, the unit of time for e.g. heartbeats and elections
const TICK: Duration = Duration::from_millis(100);

/// A Raft server
pub struct Server {
    node: Node,
    peers: HashMap<String, String>,
    node_rx: mpsc::UnboundedReceiver<Message>,
}

impl Server {

    pub async fn new(
        id: &str,
        peers: HashMap<String, String>,
        log: Log,
        state: Box<dyn State>
    ) -> Result<Self> {
        let (node_tx, node_rx) = mpsc::unbounded_channel();
        Ok(Self {
            node: Node::new(
                id,
                peers.iter().map(|(k, _)|k.to_string()).collect(),
                log,
                state,
                node_tx
            ).await?,
            peers,
            node_rx
        })
    }

    pub async fn serve(
        self,
        listener: TcpListener,
        client_rx: mpsc::UnboundedReceiver<(Request, oneshot::Sender<Result<Response>>)>
    ) -> Result<()> {
        let (tcp_in_tx, tcp_in_rx) = mpsc::unbounded_channel::<Message>();
        let (tcp_out_tx, tcp_out_rx) = mpsc::unbounded_channel::<Message>();
        let (task, tcp_receiver) = Self::tcp_receive(listener, tcp_in_tx).remote_handle();

    }

    /// Receives inbound message from peers via Tcp
    async fn tcp_receive(
        mut listener: TcpListener,
        in_tx: mpsc::UnboundedSender<Message>
    ) -> Result<()> {
        while let Some(socket) = listener.try_next().await? {
            let peer = socket.peer_addr()?;
            let peer_in_tx = in_tx.clone();
            tokio::spawn(async move {
                debug!("Raft peer {} connected", peer);
                match Self::tcp_receive_peer(socket, peer_in_tx).await {
                    Ok(()) => debug!("Raft peer {} disconnected", peer),
                    Err(err) => error!("Raft peer {} error: {}", peer, err.to_string())
                }
            });
        }
        Ok(())
    }

    async fn tcp_receive_peer(
        socket: TcpStream,
        in_tx: mpsc::UnboundedSender<Message>
    ) -> Result<()> {
        let mut stream = tokio_serde::SymmetricallyFramed::<_, Message, _>::new(
            Framed::new(socket, LengthDelimitedCodec::new()),
            tokio_serde::formats::SymmetricalBincode::<Message>::default(),
        );
        while let Some(message) = stream.try_next().await? {
            in_tx.send(message)?;
        }
        Ok(())
    }

    /// Sends outbound messages to peers via TCP
    async fn tcp_send(
        node_id: String,
        peers: HashMap<String, String>,
        mut out_rx: mpsc::UnboundedReceiver<Message>
    ) -> Result<()> {
        let mut peer_txs : HashMap<String, mpsc::Sender<Message>> = HashMap::new();
        for (id, addr) in peers.into_iter() {
            let (tx, rx) = mpsc::channel::<Message>(1000);
            peer_txs.insert(id, tx);
            tokio::spawn(Self::tcp_send_peer(addr, rx));
        }

        while let Some(mut message) = out_rx.next().await {
            if message.from == Address::Local {
                message.from = Address::Peer(node_id.clone());
            }
            let to = match &message.to {
                Address::Peers => peer_txs.keys().cloned().collect(),
                Address::Peer(peer) => vec![peer.to_string()],
                addr => {
                    error!("Received outbound message for non-TCP address {:?}", addr);
                    continue;
                }
            };
            for id in to {
                match peer_txs.get_mut(&id) {
                    Some(tx) => match tx.try_send(message.clone()) {
                        Ok(()) => {},
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            debug!("Full send buffer for peer {}, discarding message", id);
                        }
                        Err(error) => return Err(error.into())
                    }
                    None => error!("Received outbound message for unknown peer {}", id)
                }
            }
        }
        Ok(())
    }

    async fn tcp_send_peer(addr: String, mut out_rx: mpsc::Receiver<Message>) {
        loop {
            match TcpStream::connect(&addr).await {
                Ok(socket) => {
                    debug!("Connected to Raft peer {}", addr);
                    match Self::tcp_send_peer_session(socket, &mut out_rx).await {
                        Ok(()) => break,
                        Err(err) => error!("Failed to sending to Raft peer {}: {}", addr, err)
                    }
                }
                Err(err) => error!("Failed to connect to Raft peer {}: {}", addr, err)
            }
            tokio::time::delay_for(Duration::from_millis(1000)).await;
        }
        debug!("Disconnected from Raft peer {}", addr);
    }

    async fn tcp_send_peer_session(
        socket: TcpStream,
        out_rx: &mut mpsc::Receiver<Message>
    ) -> Result<()> {
        let mut stream = tokio_serde::SymmetricallyFramed::<_, Message, _>::new(
            Framed::new(socket, LengthDelimitedCodec::new()),
            tokio_serde::formats::SymmetricalBincode::<Message>::default()
        );
        while let Some(message) = out_rx.next().await {
            stream.send(message).await?;
        }
        Ok(())
    }
}