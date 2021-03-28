use crate::apache2::{
    apr_pool_t, log_error, server_rec, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::os::raw::c_int;
use std::path::Path;
use std::process;

pub extern "C" fn post_config(
    _config_pool: *mut apr_pool_t,
    _logging_pool: *mut apr_pool_t,
    _temp_pool: *mut apr_pool_t,
    server_info: *mut server_rec,
) -> c_int {
    let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
    let trace_path = Path::new(path_str.as_str());
    let mut trace_file = match File::create(&trace_path) {
        Err(why) => {
            log_error(
                cstr!(file!()),
                line!(),
                APLOG_ERR,
                -1,
                server_info,
                match CString::new(format!(
                    "Can't create trace file {}: {}",
                    trace_path.display(),
                    why
                )) {
                    Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
                    Ok(err_msg) => err_msg,
                },
            );
            return HTTP_INTERNAL_SERVER_ERROR as c_int;
        }
        Ok(file) => file,
    };
    match trace_file.write_all(b"init::post_config - start\n") {
        Err(why) => {
            log_error(
                cstr!(file!()),
                line!(),
                APLOG_ERR,
                -1,
                server_info,
                match CString::new(format!(
                    "Can't write to trace file {}: {}",
                    trace_path.display(),
                    why
                )) {
                    Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
                    Ok(err_msg) => err_msg,
                },
            );
            return HTTP_INTERNAL_SERVER_ERROR as c_int;
        }
        Ok(result) => result,
    };
    OK as c_int
}
