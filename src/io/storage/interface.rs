use crate::schema::tile::error::TileReadError;
use crate::schema::tile::identity::TileIdentity;
use crate::framework::apache2::context::HostContext;
use crate::schema::tile::tile_ref::TileRef;

use std::result::Result;


pub trait TileStorage {
    fn read_tile(
        &mut self,
        context: &HostContext,
        id: &TileIdentity,
    ) -> Result<TileRef, TileReadError>;

    fn clean_up(&mut self) -> ();
}

pub trait StorageInventory {
    fn primary_tile_store(&mut self) -> &mut dyn TileStorage;
    // TODO: add a method that returns the concrete type name
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::schema::http::encoding::ContentEncoding;
    use std::cell::RefCell;
    use std::vec::Vec;

    pub struct BlankTileStorage {
        blank_tile: RefCell<Vec<u8>>,
    }

    impl BlankTileStorage {
        pub fn new() -> BlankTileStorage {
            BlankTileStorage {
                blank_tile: RefCell::new(Vec::new())
            }
        }
    }

    impl TileStorage for BlankTileStorage {
        fn read_tile(
            &mut self,
            _context: &HostContext,
            _id: &TileIdentity,
        ) -> Result<TileRef, TileReadError> {
            Ok(
                TileRef {
                    raw_bytes: self.blank_tile.clone(),
                    begin: 0,
                    end: 0,
                    media_type: mime::IMAGE_PNG,
                    encoding: ContentEncoding::NotCompressed,
                }
            )
        }

        fn clean_up(&mut self) -> () {
        }
    }

    pub struct BlankStorageInventory {
        tile_stroage: BlankTileStorage,
    }

    impl BlankStorageInventory {
        pub fn new() -> BlankStorageInventory {
            BlankStorageInventory {
                tile_stroage: BlankTileStorage::new(),
            }
        }
    }

    impl StorageInventory for BlankStorageInventory {
        fn primary_tile_store(&mut self) -> &mut dyn TileStorage {
            &mut self.tile_stroage
        }
    }
}
