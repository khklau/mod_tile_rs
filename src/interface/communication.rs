use crate::schema::tile::identity::TileIdentity;
use crate::interface::handler::HandleContext;

use std::rc::Rc;
use std::result::Result;
use std::string::String;


#[derive(Debug, Clone)]
pub enum CommunicationError {
    TimeoutError,
    Io(Rc<std::io::Error>),
}

pub enum RenderResponse {
    NotDone,
    Done(String),
}

pub trait BidirectionalChannel {
    fn request_render(
        &mut self,
        context: &HandleContext,
        id: &TileIdentity,
    ) -> Result<RenderResponse, CommunicationError>;
}

pub struct RenderdCommunicationInventory<'i> {
    pub primary_comms: &'i mut dyn BidirectionalChannel,
}
