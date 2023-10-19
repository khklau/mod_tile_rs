use crate::schema::communication::error::ResponseWriteError;

use mime::Mime;

use std::convert::From;
use std::error::Error;
use std::fmt;
use std::option::Option;
use std::rc::Rc;
use std::str::Utf8Error;


#[derive(Debug, Clone)]
pub enum ReadError {
    Param(InvalidParameterError),
    Io(Rc<std::io::Error>),
    Utf8(Utf8Error),
}

#[derive(Debug, Clone)]
pub enum WriteError {
    RequestNotHandled, // FIXME: define the error type properly
    UnsupportedMediaType(Mime),
    ResponseWrite(ResponseWriteError),
    Io(Rc<std::io::Error>),
    Utf8(Utf8Error),
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadError::Param(err) => return Some(err),
            ReadError::Io(err) => return Some(&(**err)),
            ReadError::Utf8(err) => return Some(err),
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Param(err) => return write!(f, "{}", err),
            ReadError::Io(err) => return write!(f, "{}", err),
            ReadError::Utf8(err) => return write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for ReadError {
    fn from(error: std::io::Error) -> Self {
        return ReadError::Io(Rc::new(error));
    }
}

impl From<Utf8Error> for ReadError {
    fn from(error: Utf8Error) -> Self {
        return ReadError::Utf8(error);
    }
}

impl Error for WriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WriteError::RequestNotHandled => return None,
            WriteError::UnsupportedMediaType(_) => return None,
            WriteError::ResponseWrite(err) => return Some(err),
            WriteError::Io(err) => return Some(&(**err)),
            WriteError::Utf8(err) => return Some(err),
        }
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::RequestNotHandled => return write!(f, "FIXME"),
            WriteError::UnsupportedMediaType(mime) => return write!(f, "Unsupported tile media tyoe {}", mime.essence_str()),
            WriteError::ResponseWrite(err) => return write!(f, "{}", err),
            WriteError::Io(err) => return write!(f, "{}", err),
            WriteError::Utf8(err) => return write!(f, "{}", err),
        }
    }
}

impl From<ResponseWriteError> for WriteError {
    fn from(error: ResponseWriteError) -> Self {
        return WriteError::ResponseWrite(error);
    }
}

impl From<std::io::Error> for WriteError {
    fn from(error: std::io::Error) -> Self {
        return WriteError::Io(Rc::new(error));
    }
}

impl From<Utf8Error> for WriteError {
    fn from(error: Utf8Error) -> Self {
        return WriteError::Utf8(error);
    }
}

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
