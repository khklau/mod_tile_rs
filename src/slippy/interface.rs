use crate::apache2::request::RequestContext;
use crate::slippy::error::ParseError;
use crate::slippy::request::Request;

use std::result::Result;


#[derive(Debug)]
pub enum ParseOutcome {
    Matched(Request),
    NotMatched,
}

impl ParseOutcome {
    pub fn expect_match(self) -> Request {
        if let ParseOutcome::Matched(request) = self {
            request
        } else {
            panic!("Expected match ParseOutcome");
        }
    }
}

pub type ParseRequestResult = Result<ParseOutcome, ParseError>;
pub type ParseRequestFunc = fn(&RequestContext, &str) -> ParseRequestResult;
