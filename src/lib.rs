#[macro_use]
mod apache2 {
    #[macro_use]
    pub mod bindings;
    pub mod config;
    #[macro_use]
    pub mod logger;
    pub mod connection;
    pub mod hook;
    pub mod memory;
    pub mod virtual_host;
}
mod resource;
mod slippy {
    pub mod context;
    pub mod parser;
    pub mod request;
}
mod storage {
    pub mod file_system;
}
mod tile {
    pub mod config;
}

use crate::apache2::bindings::{
    ap_hook_child_init, ap_hook_map_to_storage, ap_hook_translate_name,
    apr_pool_t, cmd_func, cmd_how, cmd_how_TAKE1, command_rec, module,
    APR_HOOK_FIRST, APR_HOOK_MIDDLE,
    MODULE_MAGIC_COOKIE, MODULE_MAGIC_NUMBER_MAJOR, MODULE_MAGIC_NUMBER_MINOR, OR_OPTIONS,
};
use std::ptr;
use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

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
    cmds: tile_cmds.as_ptr(),
    register_hooks: Some(register_hooks),
    flags: 0,
};

#[no_mangle]
static tile_cmds: [command_rec; 1] = [
    command_rec {
        name: cstr!("LoadTileConfigFile"),
        func: cmd_func {
            take1: Some(apache2::config::load_tile_config),
        },
        cmd_data: ptr::null_mut(),
        req_override: OR_OPTIONS as i32,
        args_how: cmd_how_TAKE1 as cmd_how,
        errmsg: cstr!("load the mod_tile/renderd/mapnik shared config file"),
    }
];

#[cfg(not(test))]
#[no_mangle]
pub extern fn register_hooks(_pool: *mut apr_pool_t) {
    unsafe {
        ap_hook_child_init(
            Some(storage::file_system::initialise),
            ptr::null_mut(),
            ptr::null_mut(),
            APR_HOOK_MIDDLE as std::os::raw::c_int,
        );
        ap_hook_translate_name(
            Some(slippy::parser::parse),
            ptr::null_mut(),
            ptr::null_mut(),
            APR_HOOK_FIRST as std::os::raw::c_int,
        );
        ap_hook_map_to_storage(
            Some(resource::handle_request),
            ptr::null_mut(),
            ptr::null_mut(),
            APR_HOOK_FIRST as std::os::raw::c_int,
        );
    }
}

#[cfg(test)]
pub extern fn register_hooks(_pool: *mut apr_pool_t) {
    // this function is a no-op for tests
}
