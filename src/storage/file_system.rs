use crate::core::apache2::{
    apr_pool_t, server_rec, APLOG_ERR,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_int;
use std::path::Path;
use std::process;
use std::ptr;

#[no_mangle]
pub extern fn initialise(
    child_pool: *mut apr_pool_t,
    server_info: *mut server_rec,
) {
    if child_pool != ptr::null_mut()
        && server_info != ptr::null_mut() {
        unsafe {
            _initialise(&mut *child_pool, &mut *server_info);
        }
    }
}

fn _initialise(
    _child_pool: &mut apr_pool_t,
    server_info: &mut server_rec,
) {
    let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
    let trace_path = Path::new(path_str.as_str());
    let mut trace_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path) {
        Ok(file) => file,
        Err(why) => {
            try_log_else!((
                APLOG_ERR,
                server_info,
                format!("Can't create trace file {}: {}", trace_path.display(), why)) {
                    return
                }
            );
            return;
        },
    };
    match trace_file.write_all(b"storage::file_system::initialise - start\n") {
        Err(why) => {
            try_log_else!((
                APLOG_ERR,
                server_info,
                format!("Can't write to trace file {}: {}", trace_path.display(), why)) {
                    return
                }
            );
            return;
        },
        Ok(result) => result,
    }
    match trace_file.write_all(b"storage::file_system::initialise - finish\n") {
        Err(why) => {
            try_log_else!((
                APLOG_ERR,
                server_info,
                format!("Can't write to trace file {}: {}", trace_path.display(), why)) {
                    return
                }
            );
            return;
        },
        Ok(result) => result,
    }
}
