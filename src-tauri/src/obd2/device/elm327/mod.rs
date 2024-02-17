pub mod transport;
use log::{debug, info, trace};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    thread, time,
};
use crate::obd2::device::elm327::transport::Transport;

use super::{Error, Obd2BaseDevice, Obd2Reader, Result};

pub struct Elm327{
    device: Box<dyn Transport>,
    buffer: VecDeque<u8>,
}

impl Obd2BaseDevice for Elm327 {
    fn reset(&mut self) -> Result<()> {
        self.reset_ic()?;
        thread::sleep(time::Duration::from_millis(500));
        self.reset_protocol()?;
        Ok(())
    }

    fn send_cmd(&mut self, data: &[u8]) -> Result<()> {
        trace!("send_cmd: sending {:?}", std::str::from_utf8(data));
        self.send_serial_str(
            data.into_iter()
                .flat_map(|v| format!("{:02X}", v).chars().collect::<Vec<char>>())
                .collect::<String>()
                .as_str(),
        )
    }
}

impl Obd2Reader for Elm327 {
    fn get_line(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'\n', false)
    }

    /// Read data until the ELM327's prompt character is printed
    ///
    /// This will receive the entire OBD-II response. The prompt signifies that the ELM327 is ready
    /// for another command. If this is not called after each OBD-II command is sent, the prompt
    /// character will come out of the receive queue later and because it is not valid hex this
    /// could cause problems. If a timeout occurs, `Ok(None)` will be returned.
    fn get_response(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'>', true)
    }
}

impl Elm327 {
    fn new(transport: Box<dyn Transport>) -> Result<Self> {
        let mut result = Elm327 {
            buffer: VecDeque::new(),
            device: transport,
        };

        result.init_device()?;

        Ok(result)
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
        self.reset()?;

        Ok(())
    }

    fn reset_ic(&mut self) -> Result<()> {
        info!("Performing IC reset");
        self.send_serial_str("ATZ")?;
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
            self.cmd(&[0x01, 0x00])?
        );
        self.flush()?;
        Ok(())
    }

    fn get_until(&mut self, end_byte: u8, allow_empty: bool) -> Result<Option<Vec<u8>>> {
        const TIMEOUT: time::Duration = time::Duration::from_secs(5);

        trace!("get_until: getting until {}", end_byte);

        let mut buf = Vec::new();
        let start = time::Instant::now();
        while start.elapsed() < TIMEOUT {
            let Some(b) = self.get_byte()? else { continue };
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

        trace!(
            "get_until: got {:?} ({:?})",
            buf,
            std::str::from_utf8(buf.as_slice())
        );

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

    fn read_into_queue(&mut self) -> Result<()> {
        let mut buf = [0u8; 16];
        loop {
            let len = self.device.read(&mut buf)?;
            if len > 0 {
                self.buffer.extend(&buf[0..len]);
                trace!(
                    "read_into_queue: values {:?}",
                    std::str::from_utf8(&buf[0..len])
                );
            } else {
                trace!("read_into_queue: no values left to read");
                break;
            }
        }
        Ok(())
    }

    pub fn serial_cmd(&mut self, cmd: &str) -> Result<Option<String>> {
        self.send_serial_str(cmd)?;
        self.get_response()
            .map(|o| o.and_then(|resp| String::from_utf8(resp).ok()))
    }

    /// Function for sending a raw string, without encoding into ASCII hex
    fn send_serial_str(&mut self, data: &str) -> Result<()> {
        trace!("send_serial_str: sending {:?}", data);

        let data = data.as_bytes();

        self.device.write(data)?;
        self.device.write(b"\r\n")?;
        let line = self.get_line()?;
        if line.as_ref().is_some_and(|v| v == data) {
            Ok(())
        } else {
            Err(Error::Communication(format!(
                "send_serial_str: got {:?} instead of echoed command ({:?})",
                line, data
            )))
        }
    }
}
