use crate::apache2::{
    apr_pool_t, server_rec, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_int;
use std::path::Path;
use std::process;
use std::ptr;

#[no_mangle]
pub extern fn post_config(
    config_pool: *mut apr_pool_t,
    logging_pool: *mut apr_pool_t,
    temp_pool: *mut apr_pool_t,
    server_info: *mut server_rec,
) -> c_int {
    if config_pool != ptr::null_mut()
        && logging_pool != ptr::null_mut()
        && temp_pool != ptr::null_mut()
        && server_info != ptr::null_mut() {
        unsafe {
            return _post_config(&mut *config_pool, &mut *logging_pool, &mut *temp_pool, &mut *server_info) as c_int;
        }
    }
    else {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    }
}

fn _post_config(
    _config_pool: &mut apr_pool_t,
    _logging_pool: &mut apr_pool_t,
    _temp_pool: &mut apr_pool_t,
    server_info: &mut server_rec,
) -> u32 {
    let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
    let trace_path = Path::new(path_str.as_str());
    let mut trace_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path) {
        Err(why) => {
            try_log_else!((
                APLOG_ERR,
                server_info,
                format!("Can't create trace file {}: {}", trace_path.display(), why)) {
                    return HTTP_INTERNAL_SERVER_ERROR
                }
            );
            return HTTP_INTERNAL_SERVER_ERROR;
        }
        Ok(file) => file,
    };
    match trace_file.write_all(b"initialisation::post_config - start\n") {
        Err(why) => {
            try_log_else!((
                APLOG_ERR,
                server_info,
                format!("Can't write to trace file {}: {}", trace_path.display(), why)) {
                    return HTTP_INTERNAL_SERVER_ERROR
                }
            );
            return HTTP_INTERNAL_SERVER_ERROR;
        }
        Ok(result) => result,
    };
    OK
}
