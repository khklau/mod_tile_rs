use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::error::{ ReadError, WriteError };
use crate::schema::slippy::request::Request;

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

#[derive(Debug)]
pub enum WriteOutcome {
    Written(HttpResponse),
    NotWritten,
}

pub type WriteResponseResult = Result<WriteOutcome, WriteError>;
