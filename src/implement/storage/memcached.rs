use crate::schema::tile::error::TileReadError;
use crate::schema::tile::identity::TileIdentity;
use crate::interface::handler::{HandleContext, HandleContext2,};
use crate::interface::tile::TileRef;
use crate::interface::storage::TileStorage;
use crate::implement::storage::meta_tile::MetaTile;


// TODO
pub struct Memcached { }

impl TileStorage for Memcached {
    fn read_tile(
        &mut self,
        context: &HandleContext,
        id: &TileIdentity,
    ) -> Result<TileRef, TileReadError> {
        let path = MetaTile::identity_to_path(context.module_config, id);
        Err(
            TileReadError::NotFound(
                path.meta_tile_path,
            )
        )
    }

    fn read_tile2(
        &mut self,
        context: &HandleContext2,
        id: &TileIdentity,
    ) -> Result<TileRef, TileReadError> {
        let path = MetaTile::identity_to_path(context.module_config, id);
        Err(
            TileReadError::NotFound(
                path.meta_tile_path,
            )
        )
    }

    fn clean_up(&mut self) -> () {
    }
}
