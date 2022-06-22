use std::convert::From;
use std::error::Error;
use std::option::Option;
use std::path::PathBuf;
use std::str::Utf8Error;


#[derive(Debug)]
pub enum TileReadError {
    Io(std::io::Error),
    InvalidTile(InvalidMetaTileError),
    NotFound(PathBuf),
    OffsetOutOfBounds(TileOffsetOutOfBoundsError),
}

impl Error for TileReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TileReadError::Io(err) => return Some(err),
            TileReadError::InvalidTile(err) => return Some(err),
            TileReadError::NotFound(_) => return None,
            TileReadError::OffsetOutOfBounds(err) => return Some(err),
        }
    }
}

impl std::fmt::Display for TileReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileReadError::Io(err) => return write!(f, "{}", err),
            TileReadError::InvalidTile(err) => return write!(f, "{}", err),
            TileReadError::NotFound(path) => return write!(f, "{}", path.to_str().unwrap()),
            TileReadError::OffsetOutOfBounds(err) => return write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for TileReadError {
    fn from(error: std::io::Error) -> Self {
        return TileReadError::Io(error);
    }
}

impl From<InvalidMetaTileError> for TileReadError {
    fn from(error: InvalidMetaTileError) -> Self {
        return TileReadError::InvalidTile(error);
    }
}

impl From<TileOffsetOutOfBoundsError> for TileReadError {
    fn from(error: TileOffsetOutOfBoundsError) -> Self {
        return TileReadError::OffsetOutOfBounds(error);
    }
}

#[derive(Debug)]
pub enum InvalidMetaTileError {
    Io(std::io::Error),
    InvalidCompression(InvalidCompressionError),
    InvalidTileCount(i32),
    InvalidTileLength(u32),
}

impl Error for InvalidMetaTileError {}

impl From<std::io::Error> for InvalidMetaTileError {
    fn from(error: std::io::Error) -> Self {
        InvalidMetaTileError::Io(error)
    }
}

impl From<InvalidCompressionError> for InvalidMetaTileError {
    fn from(error: InvalidCompressionError) -> Self {
        InvalidMetaTileError::InvalidCompression(error)
    }
}

impl std::fmt::Display for InvalidMetaTileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidMetaTileError::Io(err) => return write!(f, "{}", err),
            InvalidMetaTileError::InvalidCompression(err) => return write!(f, "{}", err),
            InvalidMetaTileError::InvalidTileCount(err) => return write!(f, "{}", err),
            InvalidMetaTileError::InvalidTileLength(err) => return write!(f, "{}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidCompressionError {
    TagIsNotUtf8(Utf8Error),
    InvalidTag(String),
}

impl Error for InvalidCompressionError {}

impl From<Utf8Error> for InvalidCompressionError {
    fn from(error: Utf8Error) -> Self {
        InvalidCompressionError::TagIsNotUtf8(error)
    }
}

impl std::fmt::Display for InvalidCompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidCompressionError::TagIsNotUtf8(err) => return write!(f, "{}", err),
            InvalidCompressionError::InvalidTag(err) => return write!(f, "{}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileOffsetOutOfBoundsError {
    pub tile_offset: u32,
}

impl Error for TileOffsetOutOfBoundsError {}

impl std::fmt::Display for TileOffsetOutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid tile offset {}", self.tile_offset)
    }
}
