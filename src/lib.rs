#[macro_use]
mod apache2 {
    #[macro_use]
    pub mod bindings;
    #[macro_use]
    pub mod logger;
    pub mod connection;
    pub mod error;
    pub mod memory;
    pub mod request;
    pub mod virtual_host;
}
mod tile {
    pub mod config;
}
mod slippy {
    pub mod error;
    pub mod interface;
    pub mod parser;
    pub mod request;
    pub mod response;
}
mod handler {
    pub mod error;
    pub mod interface;
    pub mod layer;
}
mod storage {
    pub mod file_system;
}
mod analytics {
    pub mod interface;
    pub mod statistics;
}
mod tile_proxy;


use crate::apache2::bindings::{
    APR_HOOK_MIDDLE, HTTP_INTERNAL_SERVER_ERROR,
    MODULE_MAGIC_COOKIE, MODULE_MAGIC_NUMBER_MAJOR, MODULE_MAGIC_NUMBER_MINOR, OR_OPTIONS,
    ap_hook_child_init, ap_hook_handler,
    apr_pool_t, cmd_func, cmd_how, cmd_how_TAKE1, cmd_parms, command_rec,
    module, request_rec, server_rec,
};
use crate::tile_proxy::TileProxy;

use scan_fmt::scan_fmt;

use std::alloc::System;
use std::ffi::CStr;
use std::path::PathBuf;
use std::ptr;
use std::os::raw::{ c_char, c_int, c_void, };
use std::time::Duration;


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
static tile_cmds: [command_rec; 2] = [
    command_rec {
        name: cstr!("LoadTileConfigFile"),
        func: cmd_func {
            take1: Some(load_tile_config),
        },
        cmd_data: ptr::null_mut(),
        req_override: OR_OPTIONS as i32,
        args_how: cmd_how_TAKE1 as cmd_how,
        errmsg: cstr!("load the mod_tile/renderd/mapnik shared config file"),
    },
    command_rec {
        name: cstr!("ModTileRequestTimeout"),
        func: cmd_func {
            take1: Some(load_request_timeout),
        },
        cmd_data: ptr::null_mut(),
        req_override: OR_OPTIONS as i32,
        args_how: cmd_how_TAKE1 as cmd_how,
        errmsg: cstr!("tile rendering request timeout threshold in seconds"),
    },
];


#[no_mangle]
pub extern "C" fn load_tile_config(
    cmd_ptr: *mut cmd_parms,
    _: *mut c_void,
    value: *const c_char,
) -> *const c_char {
    if cmd_ptr == ptr::null_mut() {
        return cstr!("Null cmd_parms");
    }
    let command = unsafe { &mut *cmd_ptr };
    if command.server == ptr::null_mut() {
        return cstr!("Nullptr server_rec");
    }
    let record = unsafe { &mut *(command.server) };
    debug!(record, "tile_server::load_tile_config - start");
    let path_str = unsafe { CStr::from_ptr(value).to_str().unwrap() };
    let tile_server = TileProxy::find_or_create(record).unwrap();
    let mut file_path = PathBuf::new();
    file_path.push(path_str);
    match tile_server.load_tile_config(file_path) {
        Ok(_) => {
            info!(record, "tile_server::load_tile_config - loaded config from {}", path_str);
            return ptr::null();
        },
        Err(why) => {
            error!(record, "tile_server::load_tile_config - failed because {}", why);
            return cstr!("Failed to load tile config file");
        },
    };
}

#[no_mangle]
pub extern "C" fn load_request_timeout(
    cmd_ptr: *mut cmd_parms,
    _: *mut c_void,
    value: *const c_char,
) -> *const c_char {
    if cmd_ptr == ptr::null_mut() {
        return cstr!("Null cmd_parms");
    }
    let command = unsafe { &mut *cmd_ptr };
    if command.server == ptr::null_mut() {
        return cstr!("Nullptr server_rec");
    }
    let record = unsafe { &mut *(command.server) };
    debug!(record, "tile_server::load_request_timeout - start");
    let timeout_str = unsafe { CStr::from_ptr(value).to_str().unwrap() };
    let timeout_uint = match scan_fmt!(timeout_str, "{d}", i32) {
        Ok(timeout) => timeout as u64,
        Err(_) => {
            return cstr!("ModTileRequestTimeout needs an integer argument");
        },
    };
    let duration = Duration::new(timeout_uint, 0);
    let tile_server = TileProxy::find_or_create(record).unwrap();
    tile_server.set_render_timeout(&duration);
    info!(record, "tile_server::load_request_timeout - set threshold to {} seconds", timeout_uint);
    return ptr::null();
}

#[cfg(not(test))]
#[no_mangle]
pub extern fn register_hooks(_pool: *mut apr_pool_t) {
    unsafe {
        ap_hook_child_init(
            Some(initialise),
            ptr::null_mut(),
            ptr::null_mut(),
            APR_HOOK_MIDDLE as std::os::raw::c_int,
        );
        ap_hook_handler(
            Some(handle_request),
            ptr::null_mut(),
            ptr::null_mut(),
            APR_HOOK_MIDDLE as std::os::raw::c_int,
        );
    }
}

#[cfg(test)]
pub extern fn register_hooks(_pool: *mut apr_pool_t) {
    // this function is a no-op for tests
}

#[no_mangle]
pub extern "C" fn initialise(
    child_pool: *mut apr_pool_t,
    record: *mut server_rec,
) -> () {
    if child_pool != ptr::null_mut() && record != ptr::null_mut() {
        info!(record, "initialise - start");
        let server = TileProxy::find_or_create(unsafe { &mut *record }).unwrap();
        if let Err(why) = server.initialise(unsafe { &mut *record }) {
            error!(record, "initialise - failed to initialise TileServer: {}", why);
        } else {
            info!(server.record, "initialise - finish");
        };
    }
}

#[no_mangle]
pub extern "C" fn handle_request(
    record_ptr: *mut request_rec
) -> c_int {
    if record_ptr == ptr::null_mut() {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    }
    let record = &mut unsafe { *record_ptr };
    if record.server == ptr::null_mut() {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    }

    debug!(record.server, "tile_server::handle_request - start");
    let server = &mut unsafe { *(record.server) };
    let tile_server = TileProxy::find_or_create(server).unwrap();
    match tile_server.handle_request(record) {
        Ok(result) => {
            debug!(record.server, "tile_server::handle_request - request handled");
            return result;
        },
        Err(why) => {
            error!(record.server, "tile_server::handle_request - failed: {}", why);
            return HTTP_INTERNAL_SERVER_ERROR as c_int;
        },
    };
}
