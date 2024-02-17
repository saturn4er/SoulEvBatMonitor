pub type Result<T> = std::result::Result<T, Error>;

/// An error with OBD-II communication
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An I/O error in a low-level [std::io] stream operation
    #[error("IO error: `{0:?}`")]
    IO(std::io::Error),

    /// An OBD-II or interface device protocol error
    #[error("Communication error: `{0}`")]
    Communication(String),

    /// Some part of the response (described by the `&str`) was not the expected length
    #[error("Incorrect length (`{0}`): expected `{1}`, got `{2}`")]
    IncorrectResponseLength(&'static str, usize, usize),

    /// Another error occurred
    #[error("Other OBD2 error: `{0}`")]
    Other(String),
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::Other(format!("invalid data recieved: {:?}", e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Other(format!("invalid string recieved: {:?}", e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}