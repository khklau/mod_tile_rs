use crate::schema::communication::error::ResponseWriteError;

use thiserror::Error;
use mime::Mime;

use std::fmt;
use std::rc::Rc;
use std::str::Utf8Error;


#[derive(Error, Debug, Clone)]
pub enum ReadError {
    #[error("Invalid rarameter read: {0:?}")]
    Param(#[from] InvalidParameterError),
    #[error("An IO error while reading")]
    Io(#[from] Rc<std::io::Error>),
    #[error("Non Utf8 bytes were read")]
    Utf8(#[from] Utf8Error),
}

#[derive(Error, Debug, Clone)]
pub enum WriteError {
    #[error("Nothing to write when the request was not handled")]
    RequestNotHandled, // FIXME: define the error type properly
    #[error("Unsupported tile media type: {}", .0.essence_str())]
    UnsupportedMediaType(Mime),
    #[error("An error occurred while writing the response")]
    ResponseWrite(#[from] ResponseWriteError),
    #[error("IO error while writing")]
    Io(#[from] Rc<std::io::Error>),
}

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
