use std::collections::HashMap;
use crate::elm327::{
    Error, Result,
    transport::Transport
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./assets/"]
struct Assets;


pub struct CircularBuffer<T> {
    data: Vec<T>,
    index: usize,
}

impl<T> CircularBuffer<T> {
    fn new(data: Vec<T>) -> CircularBuffer<T> {
        CircularBuffer { data, index: 0 }
    }

    fn get(&mut self) -> Option<&T> {
        if self.data.is_empty() {
            None
        } else {
            let result = &self.data[self.index];
            self.index = (self.index + 1) % self.data.len();
            Some(result)
        }
    }
}

pub struct History {
    write_data: Vec<u8>,
    responses: HashMap<String, CircularBuffer<String>>,
    next_response: Vec<u8>,
}

impl History {
    pub fn default() -> Self {
        let history_file = Assets::get("history.log").unwrap();
        let history = std::str::from_utf8(history_file.data.as_ref()).unwrap();
        let mut last_write = String::from("");
        let mut last_read = String::from("");
        let mut responses: HashMap<String, CircularBuffer<String>> = HashMap::new();
        for line in history.lines() {
            if line.starts_with("write: '") {
                if last_read.len() > 0 {
                    let buffer = responses.entry(last_write).or_insert(CircularBuffer::new(vec![]));
                    buffer.data.push(last_read.clone());
                }
                last_write = "".to_string();
                last_read = "".to_string();

                let mut parsed_line = line.trim_start_matches("write: '").trim_end_matches("'").to_string();
                parsed_line = parsed_line.clone().replace("\\r", "\r").replace("\\n", "\n");
                last_write = parsed_line.clone();
            } else if line.starts_with("read: '") {
                let read = line.trim_start_matches("read: '").trim_end_matches("'");
                // println!("write: '{}', read: '{}'", write, read);
                let read = read.replace("\\r", "\r").replace("\\n", "\n");
                last_read  = (last_read.to_string() + read.as_str()).clone();
            }
        }

        Self {
            responses,
            next_response: vec![],
            write_data: vec![],
        }
    }
    fn new(responses: HashMap<String, CircularBuffer<String>>) -> Self {
        Self {
            responses: responses,
            next_response: vec![],
            write_data: vec![],
        }
    }
}

impl Transport for History {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        if !data.ends_with(b"\r\n") {
            self.write_data = data.to_vec();
            return Ok(());
        }

        let mut command = self.write_data.clone();
        command.extend(data);

        self.write_data = vec![];

        match self.responses.get_mut(String::from_utf8(command.clone())?.as_str()) {
            Some(buffer) => {
                let response = buffer.get().ok_or(Error::Communication("No response".to_string()))?;
                self.next_response = Vec::from(response.clone());

                Ok(())
            }
            None => Err(Error::Communication("No response".to_string())),
        }
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        let len = data.len();
        let response_len = self.next_response.len();
        if response_len > len {
            data.copy_from_slice(&self.next_response[0..len]);
            self.next_response = self.next_response[len..].to_vec();
            Ok(len)
        } else {
            data[0..response_len].copy_from_slice(&self.next_response);
            self.next_response = vec![];
            Ok(response_len)
        }
    }
    fn connected(&self) -> bool {
        true
    }
}