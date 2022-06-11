use crate::schema::tile::identity::TileIdentity;
use crate::interface::handler::HandleContext;
use crate::interface::storage::TileStorage;


pub struct FileSystemStorage {}

impl TileStorage for FileSystemStorage {
    fn read_tile(
        &self,
        _context: &HandleContext,
        _id: &TileIdentity,
    ) -> () {
    }
}
