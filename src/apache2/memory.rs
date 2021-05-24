use crate::apache2::bindings::{
    apr_palloc, apr_pool_t, apr_size_t, memset,
};

use std::alloc::Layout;
use std::os::raw::c_ulong;
use std::ptr;
use std::result::Result;

pub struct AllocError {}

pub fn alloc<'p, T>(pool: &'p mut apr_pool_t) -> Result<&'p mut T, AllocError> {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr_raw = apr_palloc(pool, layout.size() as apr_size_t);
        if ptr_raw == ptr::null_mut() {
            return Err(AllocError{});
        }
        else {
            let ptr_zeroed = memset(ptr_raw, 0, layout.size() as c_ulong) as *mut T;
            return Ok(&mut *ptr_zeroed)
        }
    }
}

pub struct MemoryPool<'p> {
    pub pool: &'p mut apr_pool_t,
}

impl<'p> MemoryPool<'p> {
    pub fn new(pool_ptr: *mut apr_pool_t) -> Result<Self, AllocError> {
        if pool_ptr != ptr::null_mut() {
            return Err(AllocError{});
        }
        unsafe {
            let pool_ref = &mut *pool_ptr;
            let mem_pool = MemoryPool {
                pool: pool_ref,
            };
            return Ok(mem_pool);
        }
    }

    pub fn alloc<T>(&'p mut self) -> Result<&'p mut T, AllocError> {
        return alloc(self.pool);
    }
}
