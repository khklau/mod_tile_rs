use crate::apache2::{
    apr_pool_t, server_rec, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_int;
use std::path::Path;
use std::process;

#[no_mangle]
pub extern fn post_config(
    _config_pool: *mut apr_pool_t,
    _logging_pool: *mut apr_pool_t,
    _temp_pool: *mut apr_pool_t,
    server_info: *mut server_rec,
) -> c_int {
    let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
    let trace_path = Path::new(path_str.as_str());
    let mut trace_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path) {
        Err(why) => {
            log_error!(
                server_info,
                format!("Can't create trace file {}: {}", trace_path.display(), why),
                return HTTP_INTERNAL_SERVER_ERROR as c_int
            );
            return HTTP_INTERNAL_SERVER_ERROR as c_int;
        }
        Ok(file) => file,
    };
    match trace_file.write_all(b"init::post_config - start\n") {
        Err(why) => {
            log_error!(
                server_info,
                format!("Can't write to trace file {}: {}", trace_path.display(), why),
                return HTTP_INTERNAL_SERVER_ERROR as c_int
            );
            return HTTP_INTERNAL_SERVER_ERROR as c_int;
        }
        Ok(result) => result,
    };
    OK as c_int
}
