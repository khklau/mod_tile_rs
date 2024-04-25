use crate::binding::renderd_protocol::{protoCmd, protoCmd_cmdIgnore, protocol};
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::error::HandleError;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::request::{BodyVariant, Header, ServeTileRequest, SlippyRequest,};
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::error::TileReadError;
use crate::schema::tile::identity::TileIdentity;
use crate::schema::tile::source::TileSource;
use crate::io::interface::IOContext;
use crate::framework::apache2::context::HostContext;
use crate::service::interface::ServicesContext;
use crate::service::rendering::interface::create_request;

use chrono::Utc;

use std::any::type_name;
use std::collections::HashMap;
use std::result::Result;


pub struct TileContext<'c> {
    pub host: HostContext<'c>,
    pub io: IOContext<'c>,
    pub services: ServicesContext<'c>,
}

impl<'c> TileContext<'c> {
    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host.host
    }
}


pub struct TileHandlerState {
    render_requests_by_tile_id: HashMap<TileIdentity, i32>,
}

impl TileHandlerState {
    pub fn new(config: &ModuleConfig) -> Result<TileHandlerState, InvalidConfigError> {
        let value = TileHandlerState {
            render_requests_by_tile_id: HashMap::new(),
        };
        return Ok(value);
    }

    pub fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    pub fn fetch_tile(
        &mut self,
        context: &mut TileContext,
        header: &Header,
        body: &ServeTileRequest,
    ) -> HandleRequestResult {
        let before_timestamp = Utc::now();
        let tile_id = match body {
            ServeTileRequest::V2(body) => TileIdentity {
                x: body.x,
                y: body.y,
                z: body.z,
                layer: header.layer.clone(),
            },
            ServeTileRequest::V3(body) => TileIdentity {
                x: body.x,
                y: body.y,
                z: body.z,
                layer: header.layer.clone(),
            },
        };
        // First preference is to fetch the tile from storage if it is available
        let read_result = {
            let primary_store = context.io.storage.primary_tile_store();
            primary_store.read_tile(&context.host, &tile_id)
        };
        let tile_ref = match read_result {
            Ok(tile) => tile,
            Err(TileReadError::NotFound(tile_path)) => {
                // Second preference is to render the tile
                // TODO: calculate the rendering priority
                let request = create_request(
                    &context.module_config().renderd,
                    header,
                    body,
                );
                let mut response = protocol {
                    ver: 0 as std::os::raw::c_int,
                    cmd: protoCmd_cmdIgnore,
                    x: 0 as std::os::raw::c_int,
                    y: 0 as std::os::raw::c_int,
                    z: 0 as std::os::raw::c_int,
                    xmlname: [0; 41usize],
                    mimetype: [0; 41usize],
                    options: [0; 41usize],
                };
                match context.services.rendering.tile_renderer().render_tile(
                    &mut context.io,
                    tile_id,
                    &request,
                    &mut response,
                    1,  // TODO: calculate the priority
                ) {
                    Ok(tile_ref) => tile_ref,
                    Err(err) => return HandleRequestResult {
                        before_timestamp,
                        after_timestamp: Utc::now(),
                        result: Err(HandleError::Render(err)),
                    },
                }
            },
            Err(other) => return HandleRequestResult {
                before_timestamp,
                after_timestamp: Utc::now(),
                result: Err(HandleError::TileRead(other)),
            },
        };
        let after_timestamp = Utc::now();
        let response = response::SlippyResponse {
            header: response::Header {
                mime_type: tile_ref.media_type.clone(),
                before_timestamp,
                after_timestamp,
            },
            body: response::BodyVariant::Tile(
                response::TileResponse {
                    source: TileSource::Cache,
                    age: TileAge::Fresh,
                    tile_ref,
                }
            ),
        };
        return HandleRequestResult {
            before_timestamp,
            after_timestamp,
            result: Ok(response),
        };
    }
}
