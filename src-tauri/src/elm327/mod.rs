pub mod transport;
pub mod error;

use transport::Transport;
pub use error::{Error, Result};
use log::{debug, info, trace};
use std::{
    collections::VecDeque,
    thread, time,
};

pub struct Elm327 {
    device: Box<dyn Transport>,
    buffer: VecDeque<u8>,
}


impl Elm327 {
    pub fn new(transport: Box<dyn Transport>) -> Result<Self> {
        let mut result = Elm327 {
            buffer: VecDeque::new(),
            device: transport,
        };

        result.init_device()?;

        Ok(result)
    }

    fn reset(&mut self) -> Result<()> {
        self.reset_ic()?;
        thread::sleep(time::Duration::from_millis(500));
        self.reset_protocol()?;
        Ok(())
    }

    pub fn send_cmd(&mut self, data: &[u8]) -> Result<()> {
        trace!("send_cmd: sending {:?}", std::str::from_utf8(data));
        self.send_serial_str(
            data.into_iter()
                .flat_map(|v| format!("{:02X}", v).chars().collect::<Vec<char>>())
                .collect::<String>()
                .as_str(),
        )
    }

    fn get_line(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'\n', false)
    }

    /// Read data until the ELM327's prompt character is printed
    ///
    /// This will receive the entire OBD-II response. The prompt signifies that the ELM327 is ready
    /// for another command. If this is not called after each OBD-II command is sent, the prompt
    /// character will come out of the receive queue later and because it is not valid hex this
    /// could cause problems. If a timeout occurs, `Ok(None)` will be returned.
    pub fn get_response(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'>', true)
    }

    pub fn change_transport(&mut self, transport: Box<dyn Transport>) -> Result<()> {
        self.device = transport;
        self.init_device()
    }

    fn init_device(&mut self) -> Result<()> {
        self.device.init()?;
        self.connect()?;
        self.flush()?;


        Ok(())
    }

    /// Flush the device's buffer
    pub fn flush(&mut self) -> Result<()> {
        thread::sleep(time::Duration::from_millis(500));
        self.read_into_queue()?;
        self.buffer.clear();
        Ok(())
    }

    fn connect(&mut self) -> Result<()> {
        self.serial_cmd(" ")?;
        thread::sleep(time::Duration::from_millis(500));
        self.buffer.clear();
        self.reset()?;

        Ok(())
    }

    fn reset_ic(&mut self) -> Result<()> {
        info!("Performing IC reset");
        self.send_serial_str("AT Z")?;
        debug!(
            "reset_ic: got response {:?}",
            self.get_response()?
                .as_ref()
                .map(|l| std::str::from_utf8(l.as_slice()))
        );
        Ok(())
    }

    fn reset_protocol(&mut self) -> Result<()> {
        info!("Performing protocol reset");
        debug!(
            "reset_protocol: got response {:?}",
            self.serial_cmd("ATSP0")?
        );
        debug!(
            "reset_protocol: got OBD response {:?}",
            self.send_cmd(&[0x01, 0x00])?
        );
        self.flush()?;
        Ok(())
    }
    fn get_byte(&mut self) -> Result<Option<u8>> {
        match self.buffer.pop_front() {
            Some(b'\0') => Ok(None),
            Some(b) => Ok(Some(b)),
            None => {
                self.read_into_queue()?;
                Ok(None)
            }
        }
    }

    fn get_until(&mut self, end_byte: u8, allow_empty: bool) -> Result<Option<Vec<u8>>> {
        const TIMEOUT: time::Duration = time::Duration::from_secs(5);

        trace!("get_until: getting until {}", String::from_utf8(vec![end_byte])?);

        let mut buf = Vec::new();
        let start = time::Instant::now();
        while start.elapsed() < TIMEOUT {
            let Some(b) = self.get_byte()? else {
                thread::sleep(time::Duration::from_millis(50));
                continue;
            };
            let b = match b {
                b'\r' => Some(b'\n'),
                b'\n' => None, // no push here
                _ => Some(b),
            };
            if let Some(b) = b {
                buf.push(b);
                if b == end_byte {
                    break;
                }
            }
        }

        trace!("get_until: got ({:?})", std::str::from_utf8(buf.as_slice()));

        match buf.pop() {
            Some(b) if b == end_byte => {
                if allow_empty || !buf.is_empty() {
                    Ok(Some(buf))
                } else {
                    // empty line, try again
                    self.get_until(end_byte, allow_empty)
                }
            } // we got it
            Some(f) => {
                // incomplete line read
                for b in buf.iter().rev() {
                    self.buffer.push_front(*b);
                }
                self.buffer.push_front(f);
                Ok(None)
            }
            None => Ok(None),
        }
    }


    fn read_into_queue(&mut self) -> Result<()> {
        let mut buf = [0u8; 256];
        let len = self.device.read(&mut buf)?;
        if len > 0 {
            self.buffer.extend(&buf[0..len]);
            trace!(
                "read_into_queue: values {:?}",
                std::str::from_utf8(&buf[0..len])
            );
        } else {
            trace!("read_into_queue: no values left to read");
        }
        Ok(())
    }

    pub fn serial_cmd(&mut self, cmd: &str) -> Result<String> {
        self.send_serial_str(cmd)?;
        let response = self.get_response()?;
        match response {
            Some(response) => {
                let result = String::from_utf8(response)?;
                debug!("serial_cmd: got response '{:?}'", result);
                let prefix = format!("{}\n", cmd);
                if result.starts_with(&prefix) {
                    Ok(result[prefix.len()..].to_string())
                } else {
                    Ok(result)
                }
            }
            None => Err(Error::Communication("No response".to_string())),
        }
    }

    /// Function for sending a raw string, without encoding into ASCII hex
    fn send_serial_str(&mut self, data: &str) -> Result<()> {
        trace!("send_serial_str: sending {:?}", data);

        let data = data.as_bytes();

        self.device.write(data)?;
        self.device.write(b"\r\n")?;

        Ok(())
    }
}

