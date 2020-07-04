use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::raft::log::Log;


/// The interval between leader heartbeats, in ticks
const HEARTBEAT_INTERVAL: u64 = 1;

/// The minimum election timeout, in ticks
const ELECTION_TIMEOUT_MIN: u64 = 8 * HEARTBEAT_INTERVAL;

/// The maximum election timeout, in ticks
const ELECTION_TIMEOUT_MAX: u64 = 15 * HEARTBEAT_INTERVAL;

/// Node status
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub server: String,
    pub leader: String,
    pub term: u64,
    pub node_last_index: HashMap<String, u64>,
    pub commit_index: u64,
    pub apply_index: u64,
    pub storage: String,
    pub storage_size: u64,
}

/// A Raft node with role R
pub struct RoleNode<R> {
    id: String,
    peers: Vec<String>,
    terms: u64,
    log: Log,
}