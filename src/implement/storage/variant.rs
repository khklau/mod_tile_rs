use crate::implement::storage::file_system::FileSystem;
use crate::implement::storage::memcached::Memcached;

pub enum StorageVariant {
    file_system(FileSystem),
    memcached(Memcached),
}
