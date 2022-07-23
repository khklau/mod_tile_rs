use mime::Mime;
use serde::{ Serialize, Serializer, };
use serde::ser::SerializeStruct;

use std::clone::Clone;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::rc::Weak;


#[derive(Clone, Debug)]
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

impl PartialEq for TileRef {
    fn eq(&self, other: &TileRef) -> bool {
        self.raw_bytes.as_ptr() == other.raw_bytes.as_ptr()
            && self.begin == other.begin
            && self.end == other.end
            && self.media_type == other.media_type
    }
}

impl Serialize for TileRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
            let mut state = serializer.serialize_struct("TileRef", 4)?;
            if let Some(rc) = self.raw_bytes.upgrade() {
                state.serialize_field("raw_bytes", rc.as_ref())?;
            } else {
                let empty_vec: Vec<u8> = Vec::new();
                state.serialize_field("raw_bytes", &empty_vec)?;
            }
            state.serialize_field("begin", &self.begin)?;
            state.serialize_field("end", &self.end)?;
            state.serialize_field("media_type", &self.media_type.essence_str())?;
            return state.end();
    }
}
