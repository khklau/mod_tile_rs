use crate::schema::communication::error::CommunicationError;
use crate::schema::renderd::error::RenderError;
use crate::schema::slippy::error::ReadError;
use crate::schema::tile::error::TileReadError;

use thiserror::Error;

use std::fmt;


#[derive(Error, Debug)]
pub enum HandleError {
    #[error("TODO: remove after ReadOutcome is removed")]
    Placeholder,
    #[error("Nothing to handle when the request was not read")]
    RequestNotRead(#[from] ReadError),
    #[error("{0:?}")]
    Timeout(TimeoutError),
    #[error("IO error while handling reqest")]
    Io(#[from] std::io::Error),
    #[error("Could not read tile: {0:?}")]
    TileRead(#[from] TileReadError),
    #[error("Communication error while handling the request")]
    Communication(#[from] CommunicationError),
    #[error("Tile rendering error")]
    Render(#[from] RenderError),
}

#[derive(Error, Debug)]
pub struct TimeoutError {
    pub threshold: u64,
    pub retry_after: u64,
    pub reason: String,
}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request handling timed out when threshold is {}: {}", self.threshold, self.reason)
    }
}
