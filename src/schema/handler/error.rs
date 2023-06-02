use crate::schema::communication::error::CommunicationError;
use crate::schema::slippy::error::ReadError;
use crate::schema::tile::error::TileReadError;

use std::convert::From;
use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum HandleError {
    RequestNotRead(ReadError),
    Timeout(TimeoutError),
    Io(std::io::Error),
    TileRead(TileReadError),
    Communication(CommunicationError),
}

impl Error for HandleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HandleError::RequestNotRead(err) => return Some(err),
            HandleError::Timeout(err) => return Some(err),
            HandleError::Io(err) => return Some(err),
            HandleError::TileRead(err) => return Some(err),
            HandleError::Communication(err) => return Some(err),
        }
    }
}

impl fmt::Display for HandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandleError::RequestNotRead(err) => return write!(f, "{}", err),
            HandleError::Timeout(err) => return write!(f, "{}", err),
            HandleError::Io(err) => return write!(f, "{}", err),
            HandleError::TileRead(err) => return write!(f, "{}", err),
            HandleError::Communication(err) => return write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for HandleError {
    fn from(error: std::io::Error) -> Self {
        return HandleError::Io(error);
    }
}

impl From<TileReadError> for HandleError {
    fn from(error: TileReadError) -> Self {
        return HandleError::TileRead(error);
    }
}

impl From<CommunicationError> for HandleError {
    fn from(error: CommunicationError) -> Self {
        return HandleError::Communication(error);
    }
}

#[derive(Debug)]
pub struct TimeoutError {
    pub threshold: u64,
    pub retry_after: u64,
    pub reason: String,
}

impl Error for TimeoutError {}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request handling timed out when threshold is {}: {}", self.threshold, self.reason)
    }
}
