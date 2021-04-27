#[macro_use]
mod apache2;
mod initialisation;
mod resource;
extern crate libc;

use crate::apache2::{
    ap_hook_post_config, ap_hook_map_to_storage, apr_pool_t, module, APR_HOOK_FIRST, APR_HOOK_MIDDLE,
    MODULE_MAGIC_COOKIE, MODULE_MAGIC_NUMBER_MAJOR, MODULE_MAGIC_NUMBER_MINOR,
};
use std::os::raw::c_int;
use std::ptr;

#[no_mangle]
pub extern fn register_hooks(_pool: *mut apr_pool_t) {
    unsafe {
        ap_hook_post_config(
            Some(initialisation::post_config),
            ptr::null_mut(),
            ptr::null_mut(),
            APR_HOOK_MIDDLE as c_int,
        );
        ap_hook_map_to_storage(
            Some(resource::handle_request),
            ptr::null_mut(),
            ptr::null_mut(),
            APR_HOOK_FIRST as c_int);
    }
}

#[no_mangle]
pub static mut TILE_MODULE: module = module {
    version: MODULE_MAGIC_NUMBER_MAJOR as i32,
    minor_version: MODULE_MAGIC_NUMBER_MINOR as i32,
    module_index: -1,
    name: cstr!("mod_tile_rs"),
    dynamic_load_handle: ptr::null_mut(),
    next: ptr::null_mut(),
    magic: MODULE_MAGIC_COOKIE as u64,
    rewrite_args: None,
    create_dir_config: None,
    merge_dir_config: None,
    create_server_config: None,
    merge_server_config: None,
    cmds: ptr::null(),
    register_hooks: Some(register_hooks),
    flags: 0,
};
