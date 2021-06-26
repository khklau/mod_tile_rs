#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    request_rec, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use crate::slippy::request::RequestContext;

use std::io::Write;
use std::os::raw::c_int;
use std::ptr;
use std::result::Result;

#[no_mangle]
pub extern fn handle_request(
    request_info: *mut request_rec
) -> c_int {
    if request_info != ptr::null_mut() {
        unsafe {
            log!(APLOG_ERR, (&mut *request_info).server, "resource::handle_request - start");
            let context = match RequestContext::find_or_create(&mut *request_info) {
                Ok(context) => context,
                Err(why) => {
                    log!(APLOG_ERR, (*request_info).server, format!("Failed to create RequestContext: {}", why));
                    return HTTP_INTERNAL_SERVER_ERROR as c_int;
                }
            };
            match _handle_request(context) {
                Ok(_) => {
                    log!(APLOG_ERR, context.record.server, "resource::handle_request - finish");
                    return OK as c_int
                },
                Err(why) => {
                    log!(APLOG_ERR, (*request_info).server, format!("Resource request failed: {}", why));
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
    context.worker.trace_file.borrow_mut().write_all(b"resource::_handle_request - start\n")?;
    context.worker.trace_file.borrow_mut().write_all(b"resource::_handle_request - finish\n")?;
    return Ok(());
}
