use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::io::storage::{StorageInventory, TileStorage,};
use crate::implement::io::storage::file_system::FileSystem;
use crate::implement::io::storage::variant::StorageVariant;

use std::result::Result;


pub struct StorageState {
    primary_tile_store: StorageVariant,
}

impl<'i> StorageState {
    pub fn new(
        config: &ModuleConfig
    ) -> Result<StorageState, InvalidConfigError> {
        Ok(
            StorageState {
                // TODO: pick the right variant based on config
                primary_tile_store: StorageVariant::FileSystem(
                    FileSystem::new(config)?,
                ),
            }
        )
    }
}

impl StorageInventory for StorageState {
    fn primary_tile_store(&mut self) -> &mut dyn TileStorage {
        match &mut self.primary_tile_store {
            StorageVariant::FileSystem(store) => &mut *store,
            StorageVariant::Memcached(store) => &mut *store,
        }
    }
}
