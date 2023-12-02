use crate::schema::tile::error::TileReadError;
use crate::schema::tile::identity::TileIdentity;
use crate::framework::apache2::context::RequestContext;
use crate::schema::tile::tile_ref::TileRef;
use crate::io::storage::interface::TileStorage;
use crate::io::storage::meta_tile::MetaTile;


// TODO
pub struct Memcached { }

impl TileStorage for Memcached {
    fn read_tile(
        &mut self,
        context: &RequestContext,
        id: &TileIdentity,
    ) -> Result<TileRef, TileReadError> {
        let path = MetaTile::identity_to_path(context.module_config(), id);
        Err(
            TileReadError::NotFound(
                path.meta_tile_path,
            )
        )
    }

    fn clean_up(&mut self) -> () {
    }
}
