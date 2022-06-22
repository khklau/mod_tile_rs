use mime::Mime;

use std::option::Option;
use std::rc::Weak;


pub struct TileRef {
    pub raw_bytes: Weak<Vec<u8>>,
    pub begin: usize,
    pub end: usize,
    pub media_type: Mime,
}

impl TileRef {
    pub fn with_tile<F, R>(
        &self,
        func: F,
    ) -> R
    where
        F: FnOnce(&[u8]) -> R {
        let strong_ref = self.raw_bytes.upgrade().expect("Tile has been dropped");
        let tile_bytes = &(strong_ref.as_ref()[self.begin..self.end]);
        return func(tile_bytes);
    }
}
