use std::collections::VecDeque;
use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use byteorder::{ByteOrder, BigEndian};

use mio::{Poll, PollOpt, Ready, Token};
use mio::net::TcpStream;
use mio::unix::UnixReady;

pub struct Connection {

    socket: TcpStream,
    pub token: Token,
    interest: Ready,
    send_queue: VecDeque<Rc<Vec<u8>>>,
    read_continuation: Option<u64>,
    write_continuation: bool,
}

impl Connection {
    pub fn new(socket: TcpStream, token: Token) -> Connection {
        Connection {
            socket,
            token,
            interest: Ready::from(UnixReady::hup()),
            send_queue: VecDeque::with_capacity(32),
            read_continuation: None,
            write_continuation: false
        }
    }

    pub fn readable(&mut self) -> io::Result<Option<Vec<u8>>> {
        let msg_len = match self.read_message_length()? {
            None => {
                return Ok(None);
            }
            Some(n) => n,
        };

        if msg_len == 0 {
            debug!("message is zero bytes; token={:?}", self.token);
            return Ok(None);
        }
        let msg_len = msg_len as usize;
        debug!("Expected message length is {}",msg_len);

        let mut recv_buf: Vec<u8> = Vec::with_capacity(msg_len);
        unsafe {
            recv_buf.set_len(msg_len);
        }
        let socket_ref = <TcpStream as Read>::by_ref(&mut self.socket);
        match socket_ref.take(msg_len as u64).read(&mut recv_buf) {
            Ok(n) => {
                debug!("CONN: we read {} bytes",n);
                if n < msg_len as usize {
                    return Err(Error::new(ErrorKind::InvalidData, "Did not read enough bytes"));
                }
                self.read_continuation = None;
                Ok(Some(recv_buf.to_vec()))
            }

            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    debug!("CONN: read encountered WouldBlock");
                    self.read_continuation = Some(msg_len as u64);
                    Ok(None)
                } else {
                    error!("Failed to read buffer for token: {:?}, error: {}",self.token, e);
                    Err(e)
                }
            }
        }


    }

    fn read_message_length(&mut self) -> io::Result<Option<u64>> {
        if let Some(n) = self.read_continuation {
            return Ok(Some(n));
        }

        let mut buf = [0u8;8];
        let bytes = match self.socket.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };

        if bytes < 8 {
            warn!("Found message length of {} bytes",bytes);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }
        let msg_len = BigEndian::read_u64(buf.as_ref());
        Ok(Some(msg_len))
    }

    pub fn writable(&mut self) -> io::Result<()> {
        self.send_queue.pop_front()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Could not pop send queue"))
            .and_then(|buf| {
                self.write_message(buf)
            })?;
        if self.send_queue.is_empty() {
            self.interest.remove(Ready::writable());
        }
        Ok(())
    }

    fn write_message_length(&mut self, buf: &Rc<Vec<u8>>) -> io::Result<Option<()>> {
        if self.write_continuation {
            return Ok(Some(()));
        }
        let len = buf.len();
        let mut send_buf = [0u8; 8];
        BigEndian::write_u64(&mut send_buf, len as u64);
        let len = send_buf.len();
        match self.socket.write(&send_buf) {
            Ok(n) => {
                if n < len {
                    let e = Error::new(ErrorKind::Other, "Message length failed");
                    error!("Failed to send message length for {:?}, error: {}", self.token, e);
                    Err(e)
                } else {
                    debug!("Sent message length of {} bytes",n);
                    Ok(Some(()))
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    debug!("client flusing buf; WouldBlock");
                    Ok(None)
                } else {
                    error!("Failed to send buffer for {:?}, error: {}", self.token, e);
                    Err(e)
                }
            }
        }
    }

    fn write_message(&mut self, buf: Rc<Vec<u8>>) -> io::Result<()> {
        match self.write_message_length(&buf) {
            Ok(None) => {
                self.send_queue.push_front(buf);
                return Ok(());
            }
            Ok(Some(())) => {
                ()
            },
            Err(e) => {
                error!("Failed to send buffer for {:?}, error: {}", self.token, e);
                return Err(e);
            }
        }

        let len = buf.len();
        match self.socket.write(&*buf) {
            Ok(n) => {
                debug!("CONN: we wrote {} bytes", n);
                if n < len {
                    let remaining = Rc::new(buf[n..].to_vec());
                    self.send_queue.push_front(remaining);
                    self.write_continuation = true;
                } else {
                    self.write_continuation = false;
                }
                Ok(())
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    debug!("client flushing buf; WouldBlock");
                    self.send_queue.push_front(buf);
                    self.write_continuation = true;
                    Ok(())
                } else {
                    error!("Failed to send buffer for {:?}, error: {}", self.token, e);
                    Err(e)
                }
            }
        }
    }

    pub fn send_message(&mut self, message: Rc<Vec<u8>>) -> io::Result<()> {
        trace!("connection send_message; token = {:?}", self.token);

        if self.send_queue.is_empty() {
            self.write_message(message)?;
        } else {
            self.send_queue.push_back(message);
        }
        if !self.send_queue.is_empty() && !self.interest.is_writable() {
            self.interest.insert(Ready::writable());
        }

        Ok(())
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        trace!("connection register; token = {:?}", self.token);
        self.interest.insert(Ready::readable());

        poll.register(
            &self.socket,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    pub fn reregister(&mut self, poll: &mut Poll) -> io::Result<()> {
        trace!("connection reregister; token={:?}", self.token);
        poll.reregister(
            &self.socket,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }
}
