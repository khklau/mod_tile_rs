#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    apr_pool_t, apr_status_t, conn_rec, apr_pool_userdata_get, apr_pool_userdata_set,
    APLOG_ERR, APR_BADARG, APR_SUCCESS,
};
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::alloc;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::option::Option;
use std::os::raw::c_void;
use std::ptr;

pub struct ConnectionContext<'c> {
    record: &'c mut conn_rec,
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
        }
        unsafe {
            let context = match Self::find(&mut *(record.pool), Self::get_id(record)) {
                Some(existing_context) => existing_context,
                None => {
                    let pool = &mut *(record.pool);
                    Self::create(record, pool)?
                },
            };
            log!(APLOG_ERR, context.record.base_server, "ConnectionContext::find_or_create - finish");
            return Ok(context);
        }
    }

    fn find(
        connection_pool: &'c mut apr_pool_t,
        user_data_key: CString,
    ) -> Option<&'c mut Self> {
        let mut context_ptr: *mut ConnectionContext<'c> = ptr::null_mut();
        unsafe {
            let get_result = apr_pool_userdata_get(
                &mut context_ptr as *mut *mut ConnectionContext<'c> as *mut *mut c_void,
                user_data_key.as_ptr(),
                connection_pool
            );
            if get_result == (APR_SUCCESS as i32) {
                let existing_context = &mut (*context_ptr);
                return Some(existing_context);
            } else {
                return None;
            }
        }
    }

    fn create(
        record: &'c mut conn_rec,
        connection_pool: &'c mut apr_pool_t
    ) -> Result<&'c mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, record.base_server, "ConnectionContext::create - start");
        let pool_ptr = connection_pool as *mut apr_pool_t;
        let new_context = alloc::<ConnectionContext<'c>>(connection_pool)?;
        new_context.record = record;

        let user_data_key = Self::get_id(new_context.record);
        unsafe {
            apr_pool_userdata_set(
                new_context as *mut _ as *mut c_void,
                user_data_key.as_ptr(),
                Some(drop_connection_context),
                pool_ptr
            );
        }
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
