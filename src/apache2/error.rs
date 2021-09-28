use std::any::type_name;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
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
