use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum CommunicationError {
    TimeoutError,
    Io(std::io::Error),
}

impl Error for CommunicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommunicationError::TimeoutError => return None,
            CommunicationError::Io(err) => return Some(err),
        }
    }
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommunicationError::TimeoutError => return write!(f, "TimeoutError"),
            CommunicationError::Io(err) => return write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for CommunicationError {
    fn from(error: std::io::Error) -> Self {
        return CommunicationError::Io(error);
    }
}
