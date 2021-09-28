use crate::apache2::request::RequestContext;
use crate::slippy::error::ParseError;
use crate::slippy::request::Request;

use std::option::Option;
use std::result::Result;


pub type ParseRequestFunc = fn(&RequestContext, &str) -> Result<Option<Request>, ParseError>;
