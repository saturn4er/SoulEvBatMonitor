use crate::{elm327, kia};
use serde::Serialize;

#[derive(thiserror::Error, Serialize, Debug)]
#[error("Command error: {code} - {message}")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub parameters: Option<Vec<String>>,
}

impl CommandError {
    pub fn new_internal() -> Self {
        Self {
            code: "internal_error".to_string(),
            message: "Internal error".to_string(),
            parameters: None,
        }
    }
    pub fn new_not_connected() -> Self {
        Self {
            code: "not_connected".to_string(),
            message: "Not connected".to_string(),
            parameters: None,
        }
    }
}

impl From<error_stack::Report<elm327::Error>> for CommandError {
    fn from(e: error_stack::Report<elm327::Error>) -> Self {
        match e.current_context() {
            elm327::Error::Communication | elm327::Error::Other => CommandError::new_internal(),
            elm327::Error::NotConnected => CommandError::new_not_connected(),
        }
    }
}

impl From<error_stack::Report<kia::Error>> for CommandError {
    fn from(e: error_stack::Report<kia::Error>) -> Self {
        match e.current_context() {
            kia::Error::NotConnected => CommandError::new_not_connected(),
            kia::Error::Other => CommandError::new_internal(),
        }
    }
}

impl From<error_stack::Report<elm327::transport::Error>> for CommandError {
    fn from(e: error_stack::Report<elm327::transport::Error>) -> Self {
        match e.current_context() {
            elm327::transport::Error::ConnectionFailed => CommandError {
                code: "connection_failed".to_string(),
                message: "Connection failed".to_string(),
                parameters: None,
            },
            elm327::transport::Error::NotConnected => CommandError::new_not_connected(),
            elm327::transport::Error::IO(_) | elm327::transport::Error::Other => CommandError::new_internal(),
            elm327::transport::Error::InvalidParameter(param, msg) => CommandError {
                code: "bad_parameter".to_string(),
                message: "Bad parameter".to_string(),
                parameters: Some(vec![param.clone(), msg.clone()]),
            },
        }
    }
}