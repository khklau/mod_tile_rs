use crate::slippy::context::RequestContext;
use crate::slippy::error::ParseError;
use crate::slippy::request::Request;

use crate::tile::config::TileConfig;

use std::option::Option;
use std::result::Result;


pub type ParseRequestFunc = fn(&RequestContext, &TileConfig, &str) -> Result<Option<Request>, ParseError>;
