use std::collections::HashMap;
use crate::obd2::device::elm327::transport::Transport;
use crate::obd2::device::{Error, Result};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/obd2/assets"]
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
    //map request -> []response
    responses: HashMap<Vec<u8>, CircularBuffer<Vec<u8>>>,
    next_response: Vec<u8>,
}

impl History {
    pub fn default() -> Self {
        for file_name in Assets::iter() {
            println!("{}", file_name);
        }

        Self {
            responses: HashMap::new(),
            next_response: vec![],
        }
    }
    fn new(responses: HashMap<Vec<u8>, CircularBuffer<Vec<u8>>>) -> Self {
        Self {
            responses: responses,
            next_response: vec![],
        }
    }
}

impl Transport for History {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        match self.responses.get_mut(data) {
            Some(buffer) => {
                let response = buffer.get().ok_or(Error::Communication("No response".to_string()))?;
                self.next_response = response.clone();

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