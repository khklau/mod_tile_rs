use crate::schema::communication::error::CommunicationError;

use thiserror::Error;

use std::fmt;


#[derive(Error, Debug, Clone)]
pub struct InvalidParameterError {
    pub param: String,
    pub value: String,
    pub reason: String,
}

impl fmt::Display for InvalidParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parameter {} value {} is invalid: {}", self.param, self.value, self.reason)
    }
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Invalid parameter: {0:?}")]
    InvalidParameter(#[from] InvalidParameterError),
    #[error("Error communicating with rendering service: {0:?}")]
    Communication(#[from] CommunicationError)
}
