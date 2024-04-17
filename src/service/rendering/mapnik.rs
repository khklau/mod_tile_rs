use crate::binding::renderd_protocol::protocol;
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::http::encoding::ContentEncoding;
use crate::schema::renderd::error::RenderError;
use crate::schema::tile::identity::TileIdentity;
use crate::schema::tile::tile_ref::TileRef;
use crate::io::interface::IOContext;
use crate::service::rendering::interface::TileRenderer;

use chrono::{DateTime, Duration, Utc};

use std::collections::HashMap;
use std::cell::RefCell;


pub struct Mapnik {
    _request_expiry_by_tile_id: HashMap<TileIdentity, DateTime<Utc>>,
    _render_timeout: Duration,
    response_buffer: RefCell<Vec<u8>>,  // TODO: use a buffer pool
}

impl Mapnik {
    pub fn new(config: &ModuleConfig) -> Result<Mapnik, InvalidConfigError> {
        let value = Mapnik {
            _request_expiry_by_tile_id: HashMap::new(),
            _render_timeout: Duration::from_std(
                config.renderd.render_timeout.clone()
            ).or_else(|_| {
                Err(
                    InvalidConfigError {
                        entry: String::from("render_timeout"),
                        reason: String::from("Invalid duration range"),
                    }
                )
            })?,
            response_buffer: RefCell::new(Vec::new()),
        };
        return Ok(value);
    }
}

impl TileRenderer for Mapnik {
    fn render_tile(
        &mut self,
        _io: &mut IOContext,
        _tile_id: TileIdentity,
        _request: &protocol,
        _response: &mut protocol,
        _priority: u8,
    ) -> Result<TileRef, RenderError> {
        Ok(
            TileRef {
                raw_bytes: self.response_buffer.clone(),
                begin: 0,
                end: 1,
                media_type: mime::IMAGE_PNG,
                encoding: ContentEncoding::NotCompressed,
            }
        )
    }
}


mod tests {

    use super::*;
    use std::boxed::Box;
    use std::error::Error;

    #[test]
    fn test_new() -> Result<(), Box<dyn Error>> {
        let module_config = ModuleConfig::new();
        let _value = Mapnik::new(&module_config)?;
        return Ok(())
    }
}
