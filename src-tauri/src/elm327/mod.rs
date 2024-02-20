pub mod transport;
pub mod error;

use transport::Transport;
pub use error::{Error, Result};
use log::{debug, info, trace};
use std::{
    collections::VecDeque,
    thread, time,
};
use std::io::Write;
use error_stack::{Report, ResultExt};

pub struct Elm327 {
    device: Box<dyn Transport>,
    buffer: VecDeque<u8>,
    device_name: String,
    transport_log: std::fs::File,
}

pub trait Command {
    type Response;
    fn serial_command(&self) -> String;
    fn parse_result(&self, result: String) -> Result<Self::Response>;
}

impl Elm327 {
    pub fn new(transport: Box<dyn Transport>) -> Result<Self> {
        let mut log_file = std::fs::File::create("transport.log").unwrap();
        let mut result = Elm327 {
            buffer: VecDeque::new(),
            device: transport,
            device_name: "".to_string(),
            transport_log: log_file,
        };

        result.init_device()?;

        Ok(result)
    }

    pub fn get_connected_device_name(&self) -> String {
        self.device_name.clone()
    }

    pub fn execute_command<J, T: Command<Response=J>>(&mut self, command: T)-> Result<J> {
        let response = self.serial_cmd(&command.serial_command())?;
        return command.parse_result(response)
    }


    fn reset(&mut self) -> Result<()> {
        self.reset_ic()?;
        thread::sleep(time::Duration::from_millis(500));
        self.reset_protocol()?;
        Ok(())
    }

    pub fn get_response(&mut self) -> Result<Option<Vec<u8>>> {
        let response = self.get_until(b'>', true)?;
        match &response {
            Some(response) => {
                let _ = self.transport_log.write(format!("read: '{}'\n", Self::unescape(&std::str::from_utf8(response.as_slice()).unwrap_or("").to_string())).as_bytes());
            }
            None => {
                let _ = self.transport_log.write("read: !!!!!!!ERROR!!!!!!\n".as_bytes());
            }
        }

        Ok(response)
    }

    fn init_device(&mut self) -> Result<()> {
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
        self.device.init().map_err(Elm327::map_transport_error)?;
        self.device_name = self.serial_cmd(" ")?.replace("\n", "");
        info!("connected to {}", &self.device_name);
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
            self.serial_cmd("01 00")?
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

        trace!("get_until: getting until {}", String::from_utf8(vec![end_byte]).unwrap());

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
        let len = self.device.read(&mut buf).map_err(Elm327::map_transport_error)?;
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
                let result = String::from_utf8(response.clone())
                    .change_context(Error::Communication)
                    .attach_printable(format!("can't parse response to utf8 string: {:?}", response))?;
                debug!("serial_cmd: got response '{:?}'", result);
                let prefix = format!("{}\n", cmd);
                if result.starts_with(&prefix) {
                    Ok(result[prefix.len()..].to_string())
                } else {
                    Ok(result)
                }
            }
            None => Err(Report::new(Error::Communication).attach_printable("No response received until timeout")),
        }
    }

    /// Function for sending a raw string, without encoding into ASCII hex
    fn send_serial_str(&mut self, data: &str) -> Result<()> {
        trace!("send_serial_str: sending {:?}", data);

        let _ = self.transport_log.write(
            format!("write: '{}'\n", Self::escape(&data.to_string())).as_bytes());

        let data = data.as_bytes();

        self.write(data)?;
        self.write(b"\r\n")?;


        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.device.write(data).map_err(Elm327::map_transport_error)
    }

    fn map_transport_error(e: Report<transport::Error>) -> Report<Error> {
        match e.current_context() {
            transport::Error::NotConnected => e.change_context(Error::NotConnected),
            _ => e.change_context(Error::Communication),
        }
    }
    fn escape(val: &String) -> String {
        val.replace("\r", "\\r").replace("\n", "\\n")
    }
    fn unescape(val: &String) -> String {
        val.replace("\\r", "\r").replace("\\n", "\n")
    }
}
