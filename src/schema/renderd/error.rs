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

pub enum RenderError {
    InvalidParameter(InvalidParameterError),
    Communication(CommunicationError)
}
