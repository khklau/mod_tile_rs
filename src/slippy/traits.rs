use crate::slippy::context::RequestContext;
use crate::slippy::error::ParseError;
use crate::slippy::request::Request;

use crate::tile::config::{ TileConfig, LayerConfig, };

use std::option::Option;
use std::result::Result;


pub trait RequestParser {
    fn parse(
        context: &RequestContext,
        config: &TileConfig,
        request_url: &str,
    ) -> Result<Option<Request>, ParseError>;
}

pub trait LayerRequestParser {
    fn parse(
        context: &RequestContext,
        config: &LayerConfig,
        request_url: &str,
    ) -> Result<Option<Request>, ParseError>;
}
