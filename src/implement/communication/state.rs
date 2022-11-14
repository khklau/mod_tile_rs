use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::communication::{BidirectionalChannel, CommunicationInventory,};
use crate::implement::communication::renderd_socket::RenderdSocket;


pub struct CommunicationState {
    renderd_socket: RenderdSocket,
}

impl CommunicationState {
    pub fn new(
        module_config: &ModuleConfig
    ) -> Result<CommunicationState, InvalidConfigError> {
        Ok(
            CommunicationState {
                renderd_socket: RenderdSocket::new(module_config)?
            }
        )
    }
}

impl CommunicationInventory for CommunicationState {
    fn primary_renderd_comms(&mut self) -> &mut dyn BidirectionalChannel {
        &mut self.renderd_socket
    }
}
