use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::storage::{StorageInventory, TileStorage,};
use crate::implement::storage::file_system::FileSystem;
use crate::implement::storage::variant::StorageVariant;

use std::result::Result;


pub struct StorageState {
    file_system: FileSystem,
}

impl<'i> StorageState {
    pub fn new(
        config: &ModuleConfig
    ) -> Result<StorageState, InvalidConfigError> {
        Ok(
            StorageState {
                file_system: FileSystem::new(config)?,
            }
        )
    }
}

impl StorageInventory for StorageState {
    fn primary_tile_store(&mut self) -> &mut dyn TileStorage {
        &mut self.file_system
    }
}
