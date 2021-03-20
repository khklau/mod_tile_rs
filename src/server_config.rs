#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;

use ::std::ptr;
use crate::apache2::{apr_pool_t, server_rec};


pub extern "C" fn create(
        pool: *mut apr_pool_t,
        server: *mut server_rec)
    -> *mut ::std::os::raw::c_void
{
    return ptr::null_mut()
}

pub extern "C" fn merge(
        pool: *mut apr_pool_t,
        base_conf: *mut ::std::os::raw::c_void,
        new_conf: *mut ::std::os::raw::c_void)
    -> *mut ::std::os::raw::c_void
{
    return ptr::null_mut()
}
