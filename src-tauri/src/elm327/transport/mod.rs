pub mod wifi;
pub mod history;

use super::Result;

pub trait Transport : Send + Sync {
    fn init(&mut self) -> Result<()>;
    fn write(&mut self, data: &[u8]) -> Result<()>;
    fn read(&mut self, data: &mut [u8]) -> Result<usize>;
    fn connected(&self) -> bool;
}

