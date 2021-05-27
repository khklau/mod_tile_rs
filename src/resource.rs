use crate::apache2::bindings::{
    request_rec, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_int;
use std::path::Path;
use std::process;
use std::ptr;

#[no_mangle]
pub extern fn handle_request(
    request_info: *mut request_rec
) -> c_int {
    if request_info != ptr::null_mut() {
        unsafe {
            return _handle_request(&mut *request_info) as c_int;
        }
    }
    else {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    }
}

fn _handle_request(
    request_info: &mut request_rec
) -> u32 {
    let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
    let trace_path = Path::new(path_str.as_str());
    let mut trace_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path) {
        Err(why) => {
            log!(
                APLOG_ERR,
                request_info.server,
                format!("Can't create trace file {}: {}", trace_path.display(), why)
            );
            return HTTP_INTERNAL_SERVER_ERROR;
        }
        Ok(file) => file,
    };
    match trace_file.write_all(b"resource::handle_request - start\n") {
        Err(why) => {
            log!(
                APLOG_ERR,
                request_info.server,
                format!("Can't write to trace file {}: {}", trace_path.display(), why)
            );
            return HTTP_INTERNAL_SERVER_ERROR;
        }
        Ok(result) => result,
    };
    return OK
}
