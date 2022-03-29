use crate::binding::apache2::request_rec;

use std::boxed::Box;
use std::error::Error;
use std::option::Option;
use std::result::Result;

use std::ffi::CString;


pub trait PoolStored<'p> {
    fn search_pool_key(request: &request_rec) -> CString;

    fn find(request: &'p request_rec, pool_key: &CString) -> Result<Option<&'p mut Self>, Box<dyn Error>>;

    fn new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>>;

    fn find_or_allocate_new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>> {
        let id = Self::search_pool_key(request);
        match Self::find(request, &id)? {
            Some(existing) => Ok(existing),
            None => Ok(Self::new(request)?),
        }
    }
}

pub trait Writer {
    type ElementType;

    fn write(
        &mut self,
        buffer: *const Self::ElementType,
        length: usize,
    ) -> i32;
}

pub struct Apache2Writer<'r> {
    pub record: &'r mut request_rec,
    pub writer: Option<&'r mut dyn Writer<ElementType = u8>>,
}
