use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::Duration,
};

pub struct Connection {
    stream: TcpStream,
    input: Vec<u8>,
}
impl Connection {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            input: Vec::new(),
        })
    }
    pub fn send(&mut self, value: &impl Serialize) -> io::Result<()> {
        let mut bytes = serde_json::to_vec(value)?;
        bytes.push(b'\n');
        // Bound writes so an unresponsive peer cannot freeze the terminal.
        self.stream.set_nonblocking(false)?;
        self.stream
            .set_write_timeout(Some(Duration::from_millis(250)))?;
        let result = self.stream.write_all(&bytes);
        self.stream.set_nonblocking(true)?;
        result
    }
    pub fn receive<T: DeserializeOwned>(&mut self) -> io::Result<Vec<T>> {
        let mut buffer = [0; 4096];
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "Peer disconnected",
                    ));
                }
                Ok(n) => {
                    self.input.extend_from_slice(&buffer[..n]);
                    if self.input.len() > 65536 {
                        return Err(io::Error::other("Message too large"));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        let mut messages = Vec::new();
        while let Some(end) = self.input.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = self.input.drain(..=end).collect();
            messages.push(serde_json::from_slice(&line[..end])?);
        }
        Ok(messages)
    }
}
