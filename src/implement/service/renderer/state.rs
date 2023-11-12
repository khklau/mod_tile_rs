use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::tile::identity::TileIdentity;
use crate::interface::service::renderer::TileRenderer;

use chrono::{DateTime, Duration, Utc};

use std::collections::HashMap;


pub struct RendererState {
}


pub struct TileRendererState {
    request_expiry_by_tile_id: HashMap<TileIdentity, DateTime<Utc>>,
    render_timeout: Duration,
    response_buffer: Vec<u8>,  // TODO: use a buffer pool
}

impl TileRendererState {
    pub fn new(config: &ModuleConfig) -> Result<TileRendererState, InvalidConfigError> {
        let value = TileRendererState {
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
            response_buffer: Vec::new(),
        };
        return Ok(value);
    }
}
