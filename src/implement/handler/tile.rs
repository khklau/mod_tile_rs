use crate::schema::apache2::config::ModuleConfig;
use crate::schema::handler::error::HandleError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult };
use crate::schema::slippy::request::{ BodyVariant, SlippyRequest, };
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::identity::TileIdentity;
use crate::schema::tile::source::TileSource;
use crate::interface::handler::{ HandleContext, RequestHandler, };
use crate::interface::storage::{ TileStorage, TileStorageInventory, };
use crate::implement::storage::file_system::FileSystem;
use crate::implement::storage::variant::StorageVariant;

use chrono::Utc;

use std::collections::HashMap;
use std::option::Option;


pub struct TileHandlerState {
    render_requests_by_tile_id: HashMap<TileIdentity, i32>,
    primary_store: StorageVariant,
}

impl TileHandlerState {
    pub fn new(_config: &ModuleConfig) -> TileHandlerState {
        TileHandlerState {
            render_requests_by_tile_id: HashMap::new(),
            primary_store: StorageVariant::FileSystem(
                FileSystem::new()
            ),
        }
    }
}

pub struct TileHandler<'h> {
    state: &'h mut TileHandlerState,
    storage_inventory: Option<TileStorageInventory<'h>>,
}

impl<'h> TileHandler<'h> {
    pub fn new(
        state: &'h mut TileHandlerState,
        storage_inventory: Option<TileStorageInventory<'h>>,
    ) -> TileHandler<'h> {
        TileHandler {
            state,
            storage_inventory,
        }
    }
}

impl<'h> RequestHandler for TileHandler<'h> {
    fn handle(
        &mut self,
        context: &HandleContext,
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
        let primary_store: &mut dyn TileStorage = match &mut self.storage_inventory {
            Some(inventory) => inventory.primary_store,
            None => match &mut self.state.primary_store {
                StorageVariant::FileSystem(file) => file,
                StorageVariant::Memcached(mem) => mem,
            }
        };
        let tile = match primary_store.read_tile(context, &tile_id) {
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
            header: response::Header::new(
                context.request.record,
                &tile.media_type,
            ),
            body: response::BodyVariant::Tile(
                response::TileResponse {
                    source: TileSource::Cache,
                    age: TileAge::Fresh,
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

}
