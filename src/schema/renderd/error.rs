use crate::schema::communication::error::CommunicationError;

use std::error::Error;
use std::fmt;


#[derive(Debug, Clone)]
pub struct InvalidParameterError {
    pub param: String,
    pub value: String,
    pub reason: String,
}

impl Error for InvalidParameterError {}

impl fmt::Display for InvalidParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parameter {} value {} is invalid: {}", self.param, self.value, self.reason)
    }
}

#[derive(Debug)]
pub enum RenderError {
    InvalidParameter(InvalidParameterError),
    Communication(CommunicationError)
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RenderError::InvalidParameter(err) => return Some(err),
            RenderError::Communication(err) => return Some(err),
        }
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::InvalidParameter(err) => return write!(f, "{}", err),
            RenderError::Communication(err) => return write!(f, "{}", err),
        }
    }
}

impl From<InvalidParameterError> for RenderError {
    fn from(error: InvalidParameterError) -> Self {
        return RenderError::InvalidParameter(error);
    }
}

impl From<CommunicationError> for RenderError {
    fn from(error: CommunicationError) -> Self {
        return RenderError::Communication(error);
    }
}
