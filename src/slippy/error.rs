use std::convert::From;
use std::error::Error;
use std::fmt;
use std::option::Option;
use std::str::Utf8Error;


#[derive(Debug)]
pub enum ParseError {
    Param(InvalidParameterError),
    Io(std::io::Error),
    Utf8(Utf8Error),
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::Param(err) => return Some(err),
            ParseError::Io(err) => return Some(err),
            ParseError::Utf8(err) => return Some(err),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Param(err) => return write!(f, "{}", err),
            ParseError::Io(err) => return write!(f, "{}", err),
            ParseError::Utf8(err) => return write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        return ParseError::Io(error);
    }
}

impl From<Utf8Error> for ParseError {
    fn from(error: Utf8Error) -> Self {
        return ParseError::Utf8(error);
    }
}

#[derive(Debug)]
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
