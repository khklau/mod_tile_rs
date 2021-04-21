use crate::apache2::{
    log_error, request_rec, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_int;
use std::path::Path;
use std::process;

#[no_mangle]
pub extern fn handle_request(
    request_info: *mut request_rec
) -> c_int {
    let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
    let trace_path = Path::new(path_str.as_str());
    let mut trace_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path) {
        Err(why) => {
            unsafe {
                log_error(
                    cstr!(file!()),
                    line!(),
                    APLOG_ERR,
                    -1,
                    (*request_info).server,
                    match CString::new(format!(
                        "Can't create trace file {}: {}",
                        trace_path.display(),
                        why
                    )) {
                        Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
                        Ok(err_msg) => err_msg,
                    },
                );
            }
            return HTTP_INTERNAL_SERVER_ERROR as c_int;
        }
        Ok(file) => file,
    };
    match trace_file.write_all(b"resource::handle_request - start\n") {
        Err(why) => {
            unsafe {
                log_error(
                    cstr!(file!()),
                    line!(),
                    APLOG_ERR,
                    -1,
                    (*request_info).server,
                    match CString::new(format!(
                        "Can't write to trace file {}: {}",
                        trace_path.display(),
                        why
                    )) {
                        Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
                        Ok(err_msg) => err_msg,
                    },
                );
            }
            return HTTP_INTERNAL_SERVER_ERROR as c_int;
        }
        Ok(result) => result,
    };
    return OK as c_int;
}
