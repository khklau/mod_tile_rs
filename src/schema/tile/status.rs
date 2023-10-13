use crate::schema::tile::identity::TileIdentity;

use std::time::SystemTime;


pub struct TileStatus {
    pub tile_identity: TileIdentity,
    pub size: u64,
    pub last_access_time: SystemTime,
    pub last_modification_time: SystemTime,
    pub creation_time: SystemTime,
    pub has_expired: bool,
}
