
pub type Result<T> = error_stack::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Communication error")]
    Communication,
    #[error("Not connected")]
    NotConnected,
    #[error("Other error")]
    Other,
}
