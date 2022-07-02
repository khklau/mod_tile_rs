use crate::implement::storage::file_system::FileSystem;
use crate::implement::storage::memcached::Memcached;

pub enum StorageVariant {
    FileSystem(FileSystem),
    Memcached(Memcached),
}
