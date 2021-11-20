use crate::apache2::request::RequestContext;
use crate::apache2::response::{ HttpResponse, ResponseContext };
use crate::slippy::error::{ ReadError, WriteError };
use crate::schema::slippy::request::Request;
use crate::schema::slippy::response::Response;

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

#[derive(Debug)]
pub enum WriteOutcome {
    Written(HttpResponse),
    NotWritten,
}

pub type WriteResponseResult = Result<WriteOutcome, WriteError>;
pub type WriteResponseFunc = fn(&mut ResponseContext, &Response) -> WriteResponseResult;
