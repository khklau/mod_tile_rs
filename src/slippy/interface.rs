use crate::apache2::request::RequestContext;
use crate::slippy::error::ParseError;
use crate::slippy::request::Request;

use std::option::Option;
use std::result::Result;


pub type ParseRequestResult = Result<Option<Request>, ParseError>;
pub type ParseRequestFunc = fn(&RequestContext, &str) -> ParseRequestResult;
