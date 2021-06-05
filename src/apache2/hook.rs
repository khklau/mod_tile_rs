use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct InvalidArgError {
    pub arg: String,
    pub reason: String,
}

impl Error for InvalidArgError {}

impl fmt::Display for InvalidArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Apache2 hook argument {} is invalid: {}", self.arg, self.reason)
    }
}
