use std::time::Duration;
use crate::raft::node::Node;
use std::collections::HashMap;
use crate::raft::message::{Message, Request, Response};
use crate::raft::log::Log;
use crate::error::Result;
use crate::raft::state::State;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt as _;
use tokio::sync::{mpsc, oneshot};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;
use tokio_serde::Framed;
use futures::{TryStreamExt, SinkExt};


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
            })
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