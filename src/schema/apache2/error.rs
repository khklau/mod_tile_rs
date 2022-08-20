use std::any::type_name;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct InvalidRecordError {
    pub record: String,
    pub address: usize,
    pub reason: String,
}

impl InvalidRecordError {
    pub fn new<T>(
        record: *const T,
        reason: &str,
    ) -> InvalidRecordError {
        InvalidRecordError {
            record: type_name::<T>().to_string(),
            address: record as usize,
            reason: reason.to_string(),
        }
    }
}

impl Error for InvalidRecordError {}

impl fmt::Display for InvalidRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Record {} @{} is invalid: {}", self.record, self.address, self.reason)
    }
}

#[derive(Debug, Clone)]
pub struct ResponseWriteError {
    pub error_code: i32,
}

impl Error for ResponseWriteError {}

impl fmt::Display for ResponseWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error on writing response: code {}", self.error_code)
    }
}

#[derive(Debug, Clone)]
pub struct InvalidConfigError {
    pub entry: String,
    pub reason: String,
}

impl Error for InvalidConfigError {}

impl fmt::Display for InvalidConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "In module config entry {} is invalid: {}", self.entry, self.reason)
    }
}
