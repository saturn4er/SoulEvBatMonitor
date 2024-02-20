mod wifi;
mod serial;

#[allow(unused_imports)]
pub use wifi::WiFi;
pub use serial::Serial;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Connection failed")]
    ConnectionFailed,
    #[error("Not connected")]
    NotConnected,
    #[error("IO error: `{0:?}`")]
    IO(#[from] std::io::Error),
    #[error("Invalid parameter {0}: `{1}`")]
    InvalidParameter(String, String),
    #[error("Internal error")]
    Other,
}


type Result<T> = error_stack::Result<T, Error>;

pub trait Transport : Send + Sync {
    fn init(&mut self) -> Result<()>;
    fn write(&mut self, data: &[u8]) -> Result<()>;
    fn read(&mut self, data: &mut [u8]) -> Result<usize>;
    fn connected(&self) -> bool;
}

