use crate::apache2::bindings::{
    request_rec, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, DECLINED, OK,
};
use crate::apache2::request::Request;

use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_int;
use std::path::Path;
use std::process;
use std::ptr;
use std::result::Result;

#[no_mangle]
pub extern fn translate(
    record: *mut request_rec,
) -> c_int {
    if record == ptr::null_mut() {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    }
    else {
        unsafe {
            match Request::new(&mut *record) {
                Ok(request) => match _translate(request) {
                    Ok(_) => return OK as c_int,
                    Err(err) => match err {
                        TranslateError::Param(_) => return DECLINED as c_int,
                        TranslateError::Env(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
                    },
                },
                Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
            }
        }
    }
}

#[derive(Debug)]
enum TranslateError {
    Param(InvalidParameterError),
    Env(EnvironmentError),
}

#[derive(Debug)]
struct InvalidParameterError {
    param: String,
    reason: String,
}

impl Error for InvalidParameterError {}

impl fmt::Display for InvalidParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parameter {} is invalid: {}", self.param, self.reason)
    }
}

#[derive(Debug)]
struct EnvironmentError {
    reason: String,
}

impl Error for EnvironmentError {}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Environment error: {}", self.reason)
    }
}

fn _translate(
    request: &Request,
) -> Result<CString, TranslateError> {
    let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
    let trace_path = Path::new(path_str.as_str());
    let mut trace_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path) {
        Ok(file) => file,
        Err(why) => {
            log!(
                APLOG_ERR,
                request.record.server,
                format!("Can't create trace file {}: {}", trace_path.display(), why)
            );
            return Err(TranslateError::Env(EnvironmentError { reason: format!("Can't open {}", path_str) }));
        },
    };
    match trace_file.write_all(b"slippy::request::translate - start\n") {
        Err(why) => {
            log!(
                APLOG_ERR,
                request.record.server,
                format!("Can't write to trace file {}: {}", trace_path.display(), why)
            );
            return Err(TranslateError::Env(EnvironmentError { reason: format!("Can't write to {}", path_str) }));
        },
        Ok(result) => result,
    }
    match trace_file.write_all(b"slippy::request::translate - finish\n") {
        Err(why) => {
            log!(
                APLOG_ERR,
                request.record.server,
                format!("Can't write to trace file {}: {}", trace_path.display(), why)
            );
            return Err(TranslateError::Env(EnvironmentError { reason: format!("Can't write to {}", path_str) }));
        },
        Ok(result) => result,
    }
    Ok(CString::new("blah").unwrap())
}

