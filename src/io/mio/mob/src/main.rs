extern crate byteorder;
extern crate mio;
extern crate slab;

#[macro_use]
extern crate log;
extern crate env_logger;

mod server;
mod client;
mod connection;

use env_logger::Env;
use std::net::SocketAddr;
use mio::Poll;
use mio::net::TcpListener;

use crate::server::*;


fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
    let addr = "127.0.0.1:8000".parse::<SocketAddr>()
        .expect("Failed to parse host:port string");
    let socket = TcpListener::bind(&addr).expect("Failed to bind address");
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut server = Server::new(socket);
    server.run(&mut poll).expect("Failed to run server");
}