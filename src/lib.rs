#[macro_use] mod apache2;
mod server_config;
extern crate libc;

use ::std::ptr;
use ::std::os::raw::c_char;
use crate::apache2::{apr_pool_t, module, MODULE_MAGIC_COOKIE, MODULE_MAGIC_NUMBER_MAJOR, MODULE_MAGIC_NUMBER_MINOR};


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
    create_server_config: Some(server_config::create),
    merge_server_config: Some(server_config::merge),
    cmds: ptr::null(),
    register_hooks: Some(register_hooks),
    flags: 0
};