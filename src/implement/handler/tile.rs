use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::error::HandleError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult };
use crate::schema::slippy::request::{ BodyVariant, SlippyRequest, };
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::identity::TileIdentity;
use crate::schema::tile::source::TileSource;
use crate::interface::context::{IOContext, RequestContext, ServicesContext,};
use crate::interface::handler::RequestHandler;
use crate::implement::io::communication::renderd_socket::RenderdSocket;
use crate::implement::io::storage::file_system::FileSystem;
use crate::implement::io::storage::variant::StorageVariant;

use chrono::Utc;

use std::any::type_name;
use std::collections::HashMap;
use std::result::Result;


pub struct TileHandlerState {
    render_requests_by_tile_id: HashMap<TileIdentity, i32>,
    primary_store: StorageVariant,
    primary_comms: RenderdSocket,
}

impl TileHandlerState {
    pub fn new(config: &ModuleConfig) -> Result<TileHandlerState, InvalidConfigError> {
        let value = TileHandlerState {
            render_requests_by_tile_id: HashMap::new(),
            primary_store: StorageVariant::FileSystem(
                FileSystem::new(config)?
            ),
            primary_comms: RenderdSocket::new(config)?,
        };
        return Ok(value);
    }
}

impl RequestHandler for TileHandlerState {
    fn handle(
        &mut self,
        context: &RequestContext,
        io: &mut IOContext,
        services: &mut ServicesContext,
        request: &SlippyRequest,
    ) -> HandleOutcome {
        let before_timestamp = Utc::now();
        let tile_id = match &request.body {
            BodyVariant::ServeTileV2(body) => TileIdentity {
                x: body.x,
                y: body.y,
                z: body.z,
                layer: request.header.layer.clone(),
            },
            BodyVariant::ServeTileV3(body) => TileIdentity {
                x: body.x,
                y: body.y,
                z: body.z,
                layer: request.header.layer.clone(),
            },
            _ => return HandleOutcome::Ignored,
        };
        let primary_store = io.storage.primary_tile_store();
        let tile_ref = match primary_store.read_tile(context, &tile_id) {
            Ok(tile) => tile,
            Err(err) => {
                return HandleOutcome::Processed(
                    HandleRequestResult {
                        before_timestamp,
                        after_timestamp: Utc::now(),
                        result: Err(HandleError::TileRead(err)),
                    }
                )
            },
        };
        let response = response::SlippyResponse {
            header: response::Header {
                mime_type: tile_ref.media_type.clone(),
            },
            body: response::BodyVariant::Tile(
                response::TileResponse {
                    source: TileSource::Cache,
                    age: TileAge::Fresh,
                    tile_ref,
                }
            ),
        };
        let after_timestamp = Utc::now();
        return HandleOutcome::Processed(
            HandleRequestResult {
                before_timestamp,
                after_timestamp,
                result: Ok(response),
            }
        );
    }

    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}
