use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::tile::error::TileReadError;
use crate::schema::tile::identity::TileIdentity;
use crate::interface::handler::HandleContext2;
use crate::interface::storage::TileStorage;
use crate::interface::tile::TileRef;
use crate::implement::storage::meta_tile::MetaTile;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::Into;
use std::path::PathBuf;
use std::result::Result;


const MAX_CACHE_SIZE: usize = 1024;

pub struct FileSystem {
    pub cache: HashMap<PathBuf, MetaTile>,
}

impl FileSystem {
    pub fn new(_config: &ModuleConfig) -> Result<FileSystem, InvalidConfigError> {
        Ok(
            FileSystem {
                cache: HashMap::new(),
            }
        )
    }
}

impl TileStorage for FileSystem {
    fn read_tile2(
        &mut self,
        context: &HandleContext2,
        id: &TileIdentity,
    ) -> Result<TileRef, TileReadError> {
        let path = MetaTile::identity_to_path(context.module_config, id);
        let meta_tile = MetaTile::read(&path.meta_tile_path)?;
        let cached_tile = match self.cache.entry(path.meta_tile_path.clone()) {
            Entry::Occupied(mut entry) => {
                // TODO: add cache expiry logic
                entry.insert(meta_tile);
                entry.into_mut()
            },
            Entry::Vacant(entry) => entry.insert(meta_tile),
        };
        return cached_tile.select(path.tile_offset).map_err(Into::into);
    }

    fn clean_up(&mut self) -> () {
        // TODO: implement a better cache clean up policy
        if self.cache.len() > MAX_CACHE_SIZE {
            let excess_count = MAX_CACHE_SIZE - self.cache.len();
            let excess_keys: Vec<PathBuf> = self.cache.keys().take(excess_count).cloned().collect();
            for key in &excess_keys {
                self.cache.remove(key);
            };
        }
    }
}
