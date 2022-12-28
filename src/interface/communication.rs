use crate::interface::handler::HandleContext;

use std::option::Option;
use std::result::Result;
use std::string::String;


#[derive(Debug)]
pub enum CommunicationError {
    TimeoutError,
    Io(std::io::Error),
}

impl From<std::io::Error> for CommunicationError {
    fn from(error: std::io::Error) -> Self {
        return CommunicationError::Io(error);
    }
}

pub enum RenderResponse {
    NotDone,
    Done(String),
}

pub trait BidirectionalChannel {
    fn send_blocking_request(
        &mut self,
        context: &HandleContext,
        request: &[u8],
        response_buffer: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, CommunicationError>;
}

pub struct RenderdCommunicationInventory<'i> {
    pub primary_comms: &'i mut dyn BidirectionalChannel,
}

pub trait CommunicationInventory {
    fn primary_renderd_comms(&mut self) -> &mut dyn BidirectionalChannel;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub struct EmptyResultBiChannel { }

    impl BidirectionalChannel for EmptyResultBiChannel {
        fn send_blocking_request(
            &mut self,
            _context: &HandleContext,
            _request: &[u8],
            _response_buffer: Option<Vec<u8>>,
        ) -> Result<Vec<u8>, CommunicationError> {
            Ok(Vec::new())
        }
    }

    pub struct EmptyResultCommunicationInventory {
        renderd_comms: EmptyResultBiChannel,
    }

    impl EmptyResultCommunicationInventory {
        pub fn new() -> EmptyResultCommunicationInventory {
            EmptyResultCommunicationInventory {
                renderd_comms: EmptyResultBiChannel {  }
            }
        }
    }

    impl CommunicationInventory for EmptyResultCommunicationInventory {
        fn primary_renderd_comms(&mut self) -> &mut dyn BidirectionalChannel {
            &mut self.renderd_comms
        }
    }

}
