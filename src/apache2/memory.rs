use crate::apache2::bindings::{apr_palloc, memset,};

use std::alloc::{GlobalAlloc, Layout};
use std::boxed::Box;
use std::os::raw::c_ulong;
use std::ptr;

struct MemoryPool {
    pool: *mut apr_pool_t,
}

unsafe impl GlobalAlloc for MemoryPool {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.pool != ptr::null_mut() {
            let ptr = apr_palloc(self.pool, layout.size() as apr_size_t);
            if ptr != ptr::null_mut() {
                return memset(ptr, 0, layout.size() as c_ulong) as *mut u8;
            }
        }
        return ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // freeing individual allocations is not required for memory pools
        ()
    }
}

type PoolBox<T> = Box<T, MemoryPool>;
