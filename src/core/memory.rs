use crate::binding::apache2::request_rec;

use std::boxed::Box;
use std::error::Error as StdError;
use std::option::Option;
use std::result::Result;

use std::ffi::CString;


pub trait PoolStored<'p> {
    fn search_pool_key(request: &request_rec) -> CString;

    fn find(request: &'p request_rec, pool_key: &CString) -> Result<Option<&'p mut Self>, Box<dyn StdError>>;

    fn new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn StdError>>;

    fn find_or_allocate_new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn StdError>> {
        let id = Self::search_pool_key(request);
        match Self::find(request, &id)? {
            Some(existing) => Ok(existing),
            None => Ok(Self::new(request)?),
        }
    }
}
