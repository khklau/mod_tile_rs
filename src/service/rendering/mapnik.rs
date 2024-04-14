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
use std::rc::Rc;


pub struct Mapnik {
    request_expiry_by_tile_id: HashMap<TileIdentity, DateTime<Utc>>,
    render_timeout: Duration,
    //response_buffer: Rc<Vec<u8>>,  // TODO: use a buffer pool
}

impl Mapnik {
    pub fn new(config: &ModuleConfig) -> Result<Mapnik, InvalidConfigError> {
        let value = Mapnik {
            request_expiry_by_tile_id: HashMap::new(),
            render_timeout: Duration::from_std(
                config.renderd.render_timeout.clone()
            ).or_else(|_| {
                Err(
                    InvalidConfigError {
                        entry: String::from("render_timeout"),
                        reason: String::from("Invalid duration range"),
                    }
                )
            })?,
            //response_buffer: Rc::new(Vec::new()),
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
                //raw_bytes: Rc::clone(&self.response_buffer),
                raw_bytes: Rc::new(Vec::new()),
                begin: 0,
                end: 1,
                media_type: mime::IMAGE_PNG,
                encoding: ContentEncoding::NotCompressed,
            }
        )
    }
}
