use crate::io::storage::file_system::FileSystem;
use crate::io::storage::memcached::Memcached;

pub enum StorageVariant {
    FileSystem(FileSystem),
    Memcached(Memcached),
}
