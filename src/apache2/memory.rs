use crate::apache2::bindings::{
    apr_palloc, apr_pool_t, apr_size_t, memset,
};

use std::alloc::Layout;
use std::error::Error;
use std::fmt;
use std::os::raw::c_ulong;
use std::ptr;
use std::result::Result;

#[derive(Debug)]
pub struct AllocError {
    layout: Layout,
}

impl Error for AllocError {}

impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot allocate with size {} and alignment {}", self.layout.size(), self.layout.align())
    }
}

pub fn alloc<'p, T>(pool: &'p mut apr_pool_t) -> Result<&'p mut T, AllocError> {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr_raw = apr_palloc(pool, layout.size() as apr_size_t);
        if ptr_raw == ptr::null_mut() {
            return Err(AllocError{ layout: layout });
        }
        else {
            let ptr_zeroed = memset(ptr_raw, 0, layout.size() as c_ulong) as *mut T;
            return Ok(&mut *ptr_zeroed)
        }
    }
}
