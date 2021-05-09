#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}

#[macro_export]
macro_rules! try_log_else {
    (($level: expr, $server_expr: expr, $msg_expr: expr) {$log_failure_expr: expr}) => {
        unsafe {
            crate::core::apache2::ap_log_error_(
                cstr!(file!()),
                line!() as std::os::raw::c_int,
                crate::core::apache2::APLOG_NO_MODULE as c_int,
                $level as c_int,
                -1,
                $server_expr,
                match std::ffi::CString::new($msg_expr) {
                    Err(_) => $log_failure_expr,
                    Ok(err_msg) => err_msg.as_ptr(),
                },
            );
        }
    }
}

unsafe impl Send for module {}
unsafe impl Sync for module {}

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
