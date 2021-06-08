#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    request_rec, APLOG_ERR, APLOG_NOTICE, DECLINED, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use crate::apache2::request::RequestContext;

use std::convert::From;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::io::Write;
use std::os::raw::c_int;
use std::option::Option;
use std::ptr;
use std::result::Result;

#[no_mangle]
pub extern "C" fn translate(record_ptr: *mut request_rec) -> c_int {
    if record_ptr == ptr::null_mut() {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    } else {
        unsafe {
            let mut record = *record_ptr;
            let context = match RequestContext::find_or_create(&mut record) {
                Ok(context) => context,
                Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
            };
            match _translate(context) {
                Ok(file_name) => {
                    context.file_name = Some(file_name);
                    return OK as c_int;
                },
                Err(err) => match err {
                    TranslateError::Param(err) => {
                        log!(
                            APLOG_NOTICE,
                            record.server,
                            format!("Parameter {} error: {}", err.param, err.reason)
                        );
                        return DECLINED as c_int;
                    }
                    TranslateError::Io(why) => {
                        log!(APLOG_ERR, record.server, format!("IO error: {}", why));
                        return HTTP_INTERNAL_SERVER_ERROR as c_int;
                    }
                },
            }
        }
    }
}

fn _translate(context: &RequestContext) -> Result<CString, TranslateError> {
    context.worker.trace_file.borrow_mut().write_all(b"slippy::request::translate - start\n")?;
    context.worker.trace_file.borrow_mut().write_all(b"slippy::request::translate - finish\n")?;
    Ok(CString::new("blah").unwrap())
}

#[derive(Debug)]
enum TranslateError {
    Param(InvalidParameterError),
    Io(std::io::Error),
}

impl Error for TranslateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TranslateError::Param(err) => return Some(err),
            TranslateError::Io(err) => return Some(err),
        }
    }
}

impl fmt::Display for TranslateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranslateError::Param(err) => return write!(f, "{}", err),
            TranslateError::Io(err) => return write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for TranslateError {
    fn from(error: std::io::Error) -> Self {
        return TranslateError::Io(error);
    }
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
