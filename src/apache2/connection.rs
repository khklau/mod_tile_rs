#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    apr_pool_t, apr_status_t, conn_rec, server_rec,
    APLOG_ERR, APR_BADARG, APR_SUCCESS,
};
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::{ alloc, retrieve, };
use crate::apache2::virtual_host::VirtualHostContext;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

pub struct ConnectionContext<'c> {
    pub record: &'c mut conn_rec,
    pub host: &'c mut VirtualHostContext<'c>,
}

impl<'c> ConnectionContext<'c> {

    pub fn get_id(record: &conn_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    pub fn find_or_create(record: &'c mut conn_rec) -> Result<&'c mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, record.base_server, "ConnectionContext::find_or_create - start");
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "conn_rec.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        } else if record.base_server == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "conn_rec.base_server".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        unsafe {
            let context = match retrieve(&mut *(record.pool), &(Self::get_id(record))) {
                Some(existing_context) => existing_context,
                None => {
                    let server = &mut *(record.base_server);
                    let pool = &mut *(record.pool);
                    Self::create(record, server, pool)?
                },
            };
            log!(APLOG_ERR, context.record.base_server, "ConnectionContext::find_or_create - finish");
            return Ok(context);
        }
    }

    fn create(
        record: &'c mut conn_rec,
        server: &'c mut server_rec,
        connection_pool: &'c mut apr_pool_t
    ) -> Result<&'c mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, record.base_server, "ConnectionContext::create - start");
        let new_context = alloc::<ConnectionContext<'c>>(
            connection_pool,
            &(Self::get_id(record)),
            Some(drop_connection_context),
        )?.0;
        new_context.record = record;
        new_context.host = VirtualHostContext::find_or_create(server)?;
        log!(APLOG_ERR, new_context.record.base_server, "ConnectionContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
pub unsafe extern fn drop_connection_context(connection_void: *mut c_void) -> apr_status_t {
    if connection_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let context_ptr = connection_void as *mut ConnectionContext;
    log!(APLOG_ERR, (&mut *context_ptr).record.base_server, "drop_connection_context - start");
    let context_ref = &mut *context_ptr;
    drop(context_ref);
    log!(APLOG_ERR, (&mut *context_ptr).record.base_server, "drop_connection_context - finish");
    return APR_SUCCESS as apr_status_t;
}
