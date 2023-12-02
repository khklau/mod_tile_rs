use crate::implement::io::storage::file_system::FileSystem;
use crate::implement::io::storage::memcached::Memcached;

pub enum StorageVariant {
    FileSystem(FileSystem),
    Memcached(Memcached),
}
