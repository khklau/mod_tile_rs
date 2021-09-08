use crate::apache2::bindings::{
    cmd_parms,
};
use crate::apache2::virtual_host::VirtualHostContext;
use crate::tile::config::load;

use std::ffi::CStr;
use std::os::raw::{ c_char, c_void, };
use std::path::Path;
use std::ptr;


#[no_mangle]
pub extern "C" fn load_tile_config(
    cmd_context_ptr: *mut cmd_parms,
    _: *mut c_void,
    value: *const c_char,
) -> *const c_char {
    if cmd_context_ptr == ptr::null_mut() {
        return cstr!("Null cmd_parms");
    }
    let cmd_context = unsafe { &mut *cmd_context_ptr };
    if cmd_context.server == ptr::null_mut() {
        return cstr!("Null server_rec");
    }
    let path_str = match unsafe { CStr::from_ptr(value).to_str() } {
        Ok(path) => path,
        Err(_) => return cstr!("LoadTileConfig value contains invalid UTF-8"),
    };
    let file_path = Path::new(path_str);
    let server = unsafe { &mut *(cmd_context.server) };
    let host_context = match VirtualHostContext::find_or_create(server) {
        Ok(context) => context,
        Err(_) => return cstr!("Invalid server_rec"),
    };
    let tile_config = match load(file_path) {
        Ok(tile_config) => tile_config,
        Err(_) => return cstr!("Failed to parse tile config file"),
    };
    host_context.tile_config = tile_config;
    return ptr::null();
}
