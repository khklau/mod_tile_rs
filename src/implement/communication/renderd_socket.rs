use crate::interface::communication::{
    CommunicationError, BidirectionalChannel, RenderResponse
};
use crate::interface::handler::HandleContext;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::tile::identity::TileIdentity;

use std::os::unix::net::UnixStream;
use std::path::Path;
use std::result::Result;

pub struct RenderdSocket {
    socket: UnixStream,
}

impl RenderdSocket {
    pub fn new(config: &ModuleConfig) -> Result<RenderdSocket, InvalidConfigError> {
        let path = Path::new(&config.renderd.ipc_uri);
        match UnixStream::connect(path) {
            Err(ioerr) => Err(
                InvalidConfigError {
                    entry: String::from("ipc_uri"),
                    reason: ioerr.to_string(),
                }
            ),
            Ok(socket) => Ok(
                RenderdSocket {
                    socket
                }
            )
        }
    }
}

impl BidirectionalChannel for RenderdSocket {
    fn request_render(
        &mut self,
        context: &HandleContext,
        id: &TileIdentity,
    ) -> Result<RenderResponse, CommunicationError> {
        return Ok(RenderResponse::NotDone);
    }
}
