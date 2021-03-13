#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;
use ::std::ptr;
use ::std::os::raw::c_char;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! cstr {
    ($s:expr) => (
        concat!($s, "\0") as *const str as *const [c_char] as *const c_char
    );
}


pub extern "C" fn create_server_config(
        pool: *mut apr_pool_t,
        server: *mut server_rec)
    -> *mut ::std::os::raw::c_void
{
    return ptr::null_mut()
}

pub extern "C" fn merge_server_config(
        pool: *mut apr_pool_t,
        base_conf: *mut ::std::os::raw::c_void,
        new_conf: *mut ::std::os::raw::c_void)
    -> *mut ::std::os::raw::c_void
{
    return ptr::null_mut()
}

pub extern "C" fn register_hooks(p: *mut apr_pool_t)
{
    ()
}

//const tile_cmds: [command_rec; 1] = [ command_rec{} ];

pub const tile_module: module = module {
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
    create_server_config: Some(create_server_config),
    merge_server_config: Some(merge_server_config),
    cmds: ptr::null(),
    register_hooks: Some(register_hooks),
    flags: 0
};