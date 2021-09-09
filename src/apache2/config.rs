use crate::apache2::bindings::{
    cmd_parms, server_rec,
};
use crate::apache2::virtual_host::VirtualHostContext;
use crate::tile::config::load;

use std::boxed::Box;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::{ c_char, c_void, };
use std::path::Path;
use std::ptr;
use std::result::Result;


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
    let server = unsafe { &mut *(cmd_context.server) };
    let path_str = unsafe { CStr::from_ptr(value).to_str().unwrap() };
    match _load_tile_config(server, path_str) {
        Ok(_) => return ptr::null(),
        Err(_) => return cstr!("Failed to load tile config file"),
    };
}

fn _load_tile_config(
    server: &mut server_rec,
    path_str: &str,
) -> Result<(), Box<dyn Error>> {
    let file_path = Path::new(path_str);
    let tile_config = load(file_path)?;
    let host_context = VirtualHostContext::find_or_create(server)?;
    host_context.tile_config = tile_config;
    return Ok(());
}
