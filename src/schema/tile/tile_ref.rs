use crate::schema::http::encoding::ContentEncoding;

use mime::Mime;
use serde::{ Serialize, Serializer, };
use serde::ser::SerializeStruct;

use std::clone::Clone;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::cell::RefCell;


#[derive(Clone, Debug)]
pub struct TileRef {
    pub raw_bytes: RefCell<Vec<u8>>,
    pub begin: usize,
    pub end: usize,
    pub media_type: Mime,
    pub encoding: ContentEncoding,
}

impl TileRef {
    pub fn with_tile<F, R>(
        &self,
        func: F,
    ) -> R
    where
        F: FnOnce(&[u8]) -> R {
        let tile_bytes = &self.raw_bytes.borrow()[self.begin..self.end];
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
            self.with_tile(|tile_bytes| {
                state.serialize_field("raw_bytes", tile_bytes)
            })?;
            state.serialize_field("begin", &self.begin)?;
            state.serialize_field("end", &self.end)?;
            state.serialize_field("media_type", &self.media_type.essence_str())?;
            return state.end();
    }
}
