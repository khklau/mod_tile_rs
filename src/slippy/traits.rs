use crate::slippy::context::RequestContext;
use crate::slippy::error::ParseError;
use crate::slippy::request::Request;

use std::option::Option;
use std::result::Result;


pub trait RequestParser {
    fn parse(&self, context: &RequestContext) -> Result<Option<Request>, ParseError>;
}
