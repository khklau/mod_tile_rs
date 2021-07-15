#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    request_rec, HTTP_INTERNAL_SERVER_ERROR, OK,
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
