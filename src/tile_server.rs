#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS, HTTP_INTERNAL_SERVER_ERROR, OK,
    apr_pool_t, apr_status_t, cmd_parms, process_rec, request_rec, server_rec,
};
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::{ alloc, retrieve };
use crate::slippy::context::RequestContext;
use crate::storage::file_system;
use crate::tile::config::{ TileConfig, load };

use std::any::type_name;
use std::error::Error;
use std::ffi::{ CStr, CString };
use std::io::Write;
use std::os::raw::{ c_char, c_int, c_void, };
use std::path::Path;
use std::ptr;
use std::result::Result;

struct TileServer<'s> {
    pub record: &'s mut server_rec,
    pub config: TileConfig,
}

impl<'s> TileServer<'s> {

    pub fn get_id(record: &server_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    pub fn find_or_create(record: &'s mut server_rec) -> Result<&'s mut Self, Box<dyn Error>> {
        info!(record, "TileServer::find_or_create - start");
        let proc_record = Self::access_proc_record(record.process)?;
        let context = match retrieve(
            unsafe { &mut *(proc_record.pool) },
            &(Self::get_id(record))
        ) {
            Some(existing_context) => {
                info!(record, "TileServer::find_or_create - existing found");
                existing_context
            },
            None => {
                info!(record, "TileServer::find_or_create - not found");
                let mut tile_config = TileConfig::new();
                for (_, config) in &mut (tile_config.layers) {
                    if config.hostnames.is_empty() {
                        let hostname = unsafe { CStr::from_ptr(record.server_hostname) };
                        config.hostnames.push(hostname.to_str()?.to_string());
                    }
                }
                Self::create(record, tile_config)?
            },
        };
        info!(context.record, "TileServer::find_or_create - finish");
        return Ok(context);
    }

    fn access_proc_record(process: *mut process_rec) -> Result<&'s mut process_rec, Box<dyn Error>> {
        if process == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "server_rec.process".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        let proc_record = unsafe { &mut *process };
        if proc_record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "server_rec.process.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        Ok(proc_record)
    }

    pub fn create(
        record: &'s mut server_rec,
        tile_config: TileConfig,
    ) -> Result<&'s mut Self, Box<dyn Error>> {
        info!(record, "TileServer::create - start");
        let proc_record = Self::access_proc_record(record.process)?;
        let new_server = alloc::<TileServer<'s>>(
            unsafe { &mut *(proc_record.pool) },
            &(Self::get_id(record)),
            Some(drop_tile_server),
        )?.0;
        new_server.record = record;
        new_server.config = tile_config;
        info!(new_server.record, "TileServer::create - finish");
        return Ok(new_server);
    }

}

#[no_mangle]
pub extern "C" fn drop_tile_server(server_void: *mut c_void) -> apr_status_t {
    if server_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let server_ptr = server_void as *mut TileServer;
    info!((&mut *server_ptr).record, "drop_tile_server - start");
    let server_ref = unsafe { &mut *server_ptr };
    drop(server_ref);
    info!((&mut *server_ptr).record, "drop_tile_server - finish");
    return APR_SUCCESS as apr_status_t;
}

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
        return cstr!("Null server_rec");
    }
    let record = unsafe { &mut *(command.server) };
    let path_str = unsafe { CStr::from_ptr(value).to_str().unwrap() };
    match _load_tile_config(record, path_str) {
        Ok(_) => return ptr::null(),
        Err(_) => return cstr!("Failed to load tile config file"),
    };
}

fn _load_tile_config(
    record: &mut server_rec,
    path_str: &str,
) -> Result<(), Box<dyn Error>> {
    let file_path = Path::new(path_str);
    let tile_config = load(file_path)?;
    let tile_server = TileServer::find_or_create(record)?;
    tile_server.config = tile_config;
    return Ok(());
}

#[no_mangle]
pub extern "C" fn initialise(
    child_pool: *mut apr_pool_t,
    server_info: *mut server_rec,
) -> () {
    if child_pool != ptr::null_mut() && server_info != ptr::null_mut() {
        unsafe {
            debug!(server_info, "tile_server::initialise - start");
            let server = match TileServer::find_or_create(&mut *server_info) {
                Ok(context) => {
                    file_system::initialise_file_system(child_pool, server_info);
                    context
                },
                Err(why) => {
                    info!(server_info, "Failed to create TileServer: {}", why);
                    return ();
                }
            };
            debug!(server.record, "tile_server::initialise - finish");
        }
    }
}

#[no_mangle]
pub extern "C" fn handle_request(
    request_info: *mut request_rec
) -> c_int {
    if request_info != ptr::null_mut() {
        unsafe {
            info!((&mut *request_info).server, "resource::handle_request - start");
            let context = match RequestContext::find_or_create(&mut *request_info) {
                Ok(context) => context,
                Err(why) => {
                    info!((*request_info).server, "Failed to create RequestContext: {}", why);
                    return HTTP_INTERNAL_SERVER_ERROR as c_int;
                }
            };
            match _handle_request(context) {
                Ok(_) => {
                    info!(context.record.server, "resource::handle_request - finish");
                    return OK as c_int
                },
                Err(why) => {
                    info!((*request_info).server, "Resource request failed: {}", why);
                    return HTTP_INTERNAL_SERVER_ERROR as c_int;
                },
            };
        }
    }
    else {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    }
}

fn _handle_request(
    context: &RequestContext,
) -> Result<(), std::io::Error> {
    context.get_host().trace_file.borrow_mut().write_all(b"resource::_handle_request - start\n")?;
    context.get_host().trace_file.borrow_mut().write_all(b"resource::_handle_request - finish\n")?;
    return Ok(());
}
