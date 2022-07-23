use std::clone::Clone;
use std::fmt::Debug;


#[derive(Clone, Debug)]
pub enum ContentEncoding {
    NotCompressed,
    Gzip,
}
