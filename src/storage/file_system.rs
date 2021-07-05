#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    apr_pool_t, server_rec, APLOG_ERR,
};
use crate::apache2::virtual_host::VirtualHostContext;

use std::io::Write;
use std::ptr;
use std::result::Result;

#[no_mangle]
pub extern fn initialise(
    child_pool: *mut apr_pool_t,
    server_info: *mut server_rec,
) -> () {
    if child_pool != ptr::null_mut()
        && server_info != ptr::null_mut() {
        unsafe {
            log!(APLOG_ERR, server_info, "storage::file_system::initialise - start");
            let context = match VirtualHostContext::find_or_create(&mut *server_info) {
                Ok(context) => context,
                Err(why) => {
                    log!(APLOG_ERR, server_info, format!("Failed to create VirtualHostContext: {}", why));
                    return ();
                }
            };
            match _initialise(context) {
                Ok(_) => (),
                Err(why) => {
                    log!(APLOG_ERR, server_info, format!("File system initialisation failed: {}", why));
                },
            };
            log!(APLOG_ERR, context.record, "storage::file_system::initialise - finish");
        }
    }
}

fn _initialise(
    context : &VirtualHostContext,
) -> Result<(), std::io::Error> {
    context.trace_file.borrow_mut().write_all(b"storage::file_system::initialise - start\n")?;
    context.trace_file.borrow_mut().write_all(b"storage::file_system::initialise - finish\n")?;
    Ok(())
}
