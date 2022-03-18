use crate::binding::apache2::request_rec;

use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;


pub trait PoolStored<'p> {

    fn get_id(request: &request_rec) -> CString;

    fn find_or_allocate_new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>>;

    fn new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>>;
}
