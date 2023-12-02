use crate::io::communication::interface::CommunicationInventory;
use crate::io::storage::interface::StorageInventory;


pub struct IOContext<'c> {
    pub communication: &'c mut dyn CommunicationInventory,
    pub storage: &'c mut dyn StorageInventory,
}
