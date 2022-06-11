use crate::schema::tile::identity::TileIdentity;
use crate::interface::handler::HandleContext;


pub trait TileStorage {
    fn read_tile(
        &self,
        context: &HandleContext,
        id: &TileIdentity,
    ) -> ();
}

pub struct TileStorageInventory<'i> {
    pub primary_store: &'i mut dyn TileStorage,
}
