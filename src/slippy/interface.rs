use crate::apache2::request::RequestContext;
use crate::slippy::error::ReadError;
use crate::slippy::request::Request;

use std::result::Result;


#[derive(Debug)]
pub enum ReadOutcome {
    Matched(Request),
    NotMatched,
}

#[cfg(test)]
impl ReadOutcome {
    pub fn expect_matched(self) -> Request {
        if let ReadOutcome::Matched(request) = self {
            request
        } else {
            panic!("Expected match ReadOutcome");
        }
    }
}

pub type ReadRequestResult = Result<ReadOutcome, ReadError>;
pub type ReadRequestFunc = fn(&RequestContext) -> ReadRequestResult;
