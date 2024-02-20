use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use error_stack::{Report, ResultExt};

use crate::elm327::{
    transport::Transport,
};
use super::{
    Result
};


pub struct Serial {
    socket_addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl Serial {
    pub fn new(addr: &str) -> Result<Self> {
        let mut socket_addrs = addr
            .to_socket_addrs()
            .attach_printable(format!("Invalid address: {}", addr))
            .map_err(|e| Report::new(super::Error::InvalidParameter("addr".to_string(), e.to_string())))?;

        let socket_addr = socket_addrs
            .next()
            .ok_or_else(|| Report::new(super::Error::InvalidParameter("addr".to_string(), "empty address".to_string())))?;


        Ok(Self {
            socket_addr,
            stream: None,
        })
    }
    pub fn set_write_timeout(&mut self, timeout: Duration) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream
                .set_write_timeout(Some(timeout))
                .attach_printable("Failed to set write timeout")
                .change_context(super::Error::Other)?;

            return Ok(())
        }

        return Err(Report::new(super::Error::NotConnected));
    }
}

impl Transport for Serial {
    fn init(&mut self) -> Result<()> {
        let stream = TcpStream::connect_timeout(&self.socket_addr, Duration::from_secs(2))
            .attach_printable("Can't connect to tcp socket")
            .change_context(super::Error::ConnectionFailed)?;

        stream.
            set_nonblocking(true)
            .attach_printable("Failed to set non-blocking mode")
            .change_context(super::Error::Other)?;

        self.stream = Some(stream);
        Ok(())
    }
    fn write(&mut self, data: &[u8]) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            return match stream.write_all(data) {
                Ok(_) => Ok(()),
                Err(ref e) if e.kind() == io::ErrorKind::WriteZero || e.kind() == io::ErrorKind::BrokenPipe => {
                    Err(Report::new(super::Error::NotConnected))
                }
                Err(e) => {
                    Err(Report::new(super::Error::IO(e)))
                }
            }
        }

        return Err(Report::new(super::Error::NotConnected));
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        if let Some(stream) = &mut self.stream {
            return match stream.read(data) {
                Ok(0) => {
                    self.stream = None;

                    return Err(Report::new(super::Error::NotConnected));
                }
                Ok(n) => Ok(n),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                    // Treat timeout as zero bytes read, indicating no data available
                    Ok(0)
                }
                Err(e) => Err(Report::new(super::Error::IO(e)))
            };
        }

        return Err(Report::new(super::Error::NotConnected));
    }
    fn connected(&self) -> bool {
        self.stream.is_some()
    }
}