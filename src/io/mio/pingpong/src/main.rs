use bytes::{Buf, Take};
use mio::tcp::*;
use mio::util::Slab;
use mio::{TryRead, TryWrite};
use std::io::Cursor;
use std::mem;

const SERVER: mio::Token = Token(0);
const MAX_LINE: usize = 128;

struct Pong {
    server: TcpListener,
    connections: Slab<Connection>,
}

impl Pong {
    fn new(server: TcpListener) -> Pong {
        // Token `0` is reserved for the server socket, Tokens 1+ are used for 
        // client connections. The slab is initialized to return Tokens starting at 1
        let slab = Slab::new_starting_at(mio::Token(1), 1024);

        Pong {
            server: server,
            connections: slab,
        }
    }
}

#[derive(Debug)]
struct Connection {
    socket: TcpStream,
    token: mio::Token,
    state: State,
}

#[derive(Debug)]
enum State {
    Reading(Vec<u8>),
    Writing(Take<Cursor<Vec<u8>>>),
    Closed,
}

impl State {
    fn mut_read_buf (&mut self) -> &mut Vec<u8> {
        match *self {
            State::Reading(ref mut buf) => buf,
            _ => panic!("connection not in reading state"),
        }
    }

    fn read_buf(&self) -> &[u8] {
        match *self {
            State::Reading(ref buf) => buf,
            _ => panic!("connection not in reading state"),
        }
    }

    fn write_buf(&self) -> &[u8] {
        match *self {
            State::Writing(ref buf) => buf,
            _ => panic!("connection not in writing state"),
        }
    }

    fn mut_write_buf(&mut buf) -> &mut Take<Cursor<Vec<u8>>> {
        match *self {
            State::Writing(ref mut buf) => buf,
            _ => panic!("connection not in writing state"),
        }
    }
}