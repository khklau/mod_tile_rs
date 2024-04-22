use thiserror::Error;

use std::path::PathBuf;
use std::str::Utf8Error;


#[derive(Error, Debug)]
pub enum TileReadError {
    #[error("An IO error while reading a tile")]
    Io(#[from] std::io::Error),
    #[error("Invalid meta tile")]
    InvalidTile(#[from] InvalidMetaTileError),
    #[error("Meta tile not found at path: {0:?}")]
    NotFound(PathBuf),
    #[error("Meta tile contains offset that is out of bounds: {0:?}")]
    OffsetOutOfBounds(#[from] TileOffsetOutOfBoundsError),
}

#[derive(Error, Debug)]
pub enum InvalidMetaTileError {
    #[error("An IO error while reading a meta tile")]
    Io(#[from] std::io::Error),
    #[error("Meta tile is using an unsupported compression algorithm")]
    InvalidCompression(#[from] InvalidCompressionError),
    #[error("Invalid tile count found in meta tile: {0}")]
    InvalidTileCount(i32),
    #[error("Invalid tile length found in meta tile: {0}")]
    InvalidTileLength(u32),
}

#[derive(Error, Debug, Clone)]
pub enum InvalidCompressionError {
    #[error("Tag in meta tile is not Utf8")]
    TagIsNotUtf8(#[from] Utf8Error),
    #[error("Invalid tag in meta tile: {0}")]
    InvalidTag(String),
}

#[derive(Error, Debug, Clone)]
pub struct TileOffsetOutOfBoundsError {
    pub tile_offset: u32,
}

impl std::fmt::Display for TileOffsetOutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid tile offset {}", self.tile_offset)
    }
}
