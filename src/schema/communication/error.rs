use thiserror::Error;

use std::fmt;


#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("Timeout during communication")]
    TimeoutError,
    #[error("IO error during communication")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug, Clone)]
pub struct ResponseWriteError {
    pub error_code: i32,
}

impl fmt::Display for ResponseWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error on writing response: code {}", self.error_code)
    }
}
