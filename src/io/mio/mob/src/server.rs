use std::io::{self, ErrorKind};
use std::rc::Rc;

use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::net::TcpListener;
use mio::unix::UnixReady;

use slab;
use crate::connection::Connection;

type Slab<T> = slab::Slab<T, Token>;

pub struct Server {
    socket: TcpListener,
    token: Token,
    conns: Slab<Connection>,
    events: Events
}

impl Server {

    pub fn new(socket: TcpListener) -> Server {
        Server {
            socket,
            token: Token(10_000_000),
            conns: Slab::with_capacity(128),
            events: Events::with_capacity(1024)
        }
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {
        self.register(poll)?;
        info!("Server run loop starting...");
        loop {
            let cnt = poll.poll(&mut self.events, None)?;
            trace!("processing events... cnt={}, len={}", cnt,self.events.len());

            for i in 0..cnt {
                let event = self.events.get(i).ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "Failed to get event")
                })?;
                trace!("event={:?}; idx={:?}", event, i);
                self.ready(poll, event.token(), event.readiness());
            }
        }
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        poll.register(
            &self.socket,
            self.token,
            Ready::readable(),
            PollOpt::edge()
        ).or_else(|e| {
            error!("Failed to register server {:?}, {:?}",self.token, e);
            Err(e)
        })
    }

    fn remove_token(&mut self, token: Token) {
        match self.conns.remove(token) {
            Some(_c) => {
                debug!("reset connection; token={:?}", token);
            }
            None => {
                warn!("Unable to remove connection for {:?}", token);
            }
        }
    }

    fn ready(&mut self, poll: &mut Poll, token: Token, event: Ready){
        debug!("{:?} event = {:?}", token, event);

        if self.token != token && !self.conns.contains(token) {
            debug!("Failed to find connection for {:?}", token);
            return;
        }
        let event = UnixReady::from(event);
        if event.is_error() {
            warn!("Error event for {:?}", token);
            self.remove_token(token);
            return;
        }
        if event.is_hup() {
            trace!("Hup event for {:?}", token);
            self.remove_token(token);
            return;
        }

        let event = Ready::from(event);
        if event.is_writable() {
            trace!("Write event for {:?}", token);
            assert_ne!(self.token, token, "Received writable event for Server");
            match self.connection(token).writable() {
                Ok(()) => {},
                Err(e) => {
                    warn!("Write event failed for {:?}, {:?}", token, e);
                    self.remove_token(token);
                    return;
                }
            }
        }

        if event.is_readable() {
            trace!("Read event for {:?}", token);
            if self.token == token {
                self.accept(poll);
            } else {
                match self.readable(token) {
                    Ok(()) => {},
                    Err(e) => {
                        warn!("Read event failed for {:?}: {:?}", token, e);
                        self.remove_token(token);
                        return;
                    }
                }
            }
        }

        if self.token!=token {
            match self.connection(token).reregister(poll) {
                Ok(()) => {},
                Err(e) => {
                    warn!("Reregister failed {:?}", e);
                    self.remove_token(token);
                    return;
                }
            }
        }
    }

    fn accept(&mut self, poll: &mut Poll) {
        debug!("server accepting new socket");
        loop {
            let socket = match self.socket.accept() {
                Ok((socket, _)) => socket,
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        debug!("accept encountered WouldBlock");
                    } else {
                        error!("Failed to accept new socket, {:?}", e);
                    }
                    return;
                }
            };

            let token = match self.conns.vacant_entry() {
                Some(entry) => {
                    let c = Connection::new(socket, entry.index());
                    entry.insert(c).index()
                }
                None => {
                    error!("Failed to insert connection into slab");
                    return;
                }
            };

            debug!("registering {:?} with poller", token);
            match self.connection(token).register(poll) {
                Ok(_) => {},
                Err(e) => {
                    error!("Failed to register {:?} connection with poller, {:?}", token, e);
                    self.remove_token(token);
                }
            }
        }
    }

    fn readable(&mut self, token: Token) -> io::Result<()> {
        debug!("server conn readable; token = {:?}", token);
        while let Some(message) = self.connection(token).readable()? {
            let rc_message = Rc::new(message);
            for c in self.conns.iter_mut() {
                c.send_message(rc_message.clone())?
            }
        }
        Ok(())
    }


    fn connection(&mut self, token: Token) -> &mut Connection {
        &mut self.conns[token]
    }
}