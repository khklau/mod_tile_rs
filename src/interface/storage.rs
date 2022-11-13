use crate::schema::tile::error::TileReadError;
use crate::schema::tile::identity::TileIdentity;
use crate::interface::handler::HandleContext;
use crate::interface::tile::TileRef;

use std::result::Result;


pub trait TileStorage {
    fn read_tile(
        &mut self,
        context: &HandleContext,
        id: &TileIdentity,
    ) -> Result<TileRef, TileReadError>;

    fn clean_up(&mut self) -> ();
}

pub struct TileStorageInventory<'i> {
    pub primary_store: &'i mut dyn TileStorage,
}

pub trait StorageInventory {
    fn primary_tile_store(&mut self) -> &mut dyn TileStorage;
}
