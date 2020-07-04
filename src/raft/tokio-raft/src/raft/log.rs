use crate::error::Result;

/// A replicated log entry
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// The index of the entry
    pub index: u64,
    /// The term in which the entry was added
    pub term: u64,
    /// The state machine command. None is used to commit noops during leader election
    pub command: Option<Vec<u8>>
}

/// A metadata key
#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    TermVote
}

impl Key {
    fn encode(&self) -> Vec<u8> {
        match self {
            Self::TermVote => vec![0x00]
        }
    }
}

/// A log scan
pub type Scan<'a> = Box::<dyn Iterator<Item = Result<Entry>> + 'a>;

/// The replicated Raft log
pub struct Log {
    pub store: Box<dyn log::Store>,
}