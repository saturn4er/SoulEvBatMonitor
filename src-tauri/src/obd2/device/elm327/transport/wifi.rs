use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use crate::obd2::device::elm327::transport::Transport;
use crate::obd2::device::Error;

pub struct WiFi {
    stream: TcpStream,
    connected: bool,
}

impl WiFi {
    fn new(addr: &str) -> crate::obd2::device::Result<Self> {
        let socketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(Error::Communication("Invalid address".to_string()))?;
        // connect to the ip and port
        let mut stream = TcpStream::connect_timeout(&socketAddr, Duration::from_secs(2))?;

        Ok(Self {
            stream: stream,
            connected: false,
        })
    }
}

impl Transport for WiFi {
    fn init(&mut self) -> crate::obd2::device::Result<()> {
        Ok(())
    }
    fn write(&mut self, data: &[u8]) -> crate::obd2::device::Result<()> {
        match self.stream.write_all(data) {
            Ok(_) => Ok(()),
            Err(e) => {
                self.connected = false;

                Err(Error::IO(e))
            },
        }
    }

    fn read(&mut self, data: &mut [u8]) -> crate::obd2::device::Result<usize> {
        match self.stream.read(data) {
            Ok(0) => {
                self.connected = false;

                return Err(Error::Communication("Connection was closed".to_string()))
            },
            Ok(n) => Ok(n),
            Err(e) => Err(Error::IO(e)),
        }
    }
    fn connected(&self) -> bool {
        self.connected
    }
}