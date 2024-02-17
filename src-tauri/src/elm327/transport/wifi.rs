use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use log::trace;
use crate::elm327::{
    transport::Transport,
    Error, Result
};


pub struct WiFi {
    stream: TcpStream,
    connected: bool,
}

impl WiFi {
    pub fn new(addr: &str) -> Result<Self> {
        let socket_addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(Error::Communication("Invalid address".to_string()))?;
        // connect to the ip and port
        let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(2))?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream: stream,
            connected: false,
        })
    }
    pub fn set_write_timeout(&mut self, timeout: Duration) -> Result<()> {
        self.stream.set_write_timeout(Some(timeout))?;
        Ok(())
    }
}

impl Transport for WiFi {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn write(&mut self, data: &[u8]) -> Result<()> {
        match self.stream.write_all(data) {
            Ok(_) => Ok(()),
            Err(e) => {
                self.connected = false;

                Err(Error::IO(e))
            },
        }
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        trace!("wifi read: reading");

        match self.stream.read(data) {
            Ok(0) => {
                self.connected = false;

                return Err(Error::Communication("Connection was closed".to_string()))
            },
            Ok(n) => Ok(n),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                // Treat timeout as zero bytes read, indicating no data available
                Ok(0)
            }
            Err(e) => Err(Error::IO(e)),
        }
    }
    fn connected(&self) -> bool {
        self.connected
    }
}