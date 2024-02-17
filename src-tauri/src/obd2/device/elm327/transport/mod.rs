pub mod wifi;
pub mod history;

use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use crate::obd2::device::Result;

pub trait Transport {
    fn init(&mut self) -> Result<()>;
    fn write(&mut self, data: &[u8]) -> Result<()>;
    fn read(&mut self, data: &mut [u8]) -> Result<usize>;
    fn connected(&self) -> bool;
}

