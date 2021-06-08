use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS,
    apr_pool_t, apr_pool_userdata_get, apr_pool_userdata_set,
    apr_status_t, conn_rec, request_rec, server_rec,
};
use crate::apache2::connection::ConnectionContext;
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::alloc;
use crate::apache2::worker::WorkerContext;

use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::{c_char, c_void,};
use std::result::Result;
use std::ptr;

pub struct RequestContext<'r> {
    pub record: &'r mut request_rec,
    pub worker: &'r mut WorkerContext<'r>,
    pub connection: &'r mut ConnectionContext<'r>,
    pub file_name: Option<CString>,
}

impl<'r> RequestContext<'r> {
    const USER_DATA_KEY: *const c_char = cstr!(module_path!());

    pub fn find_or_create(record: &'r mut request_rec) -> Result<&'r mut Self, Box<dyn Error>> {
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        } else if record.server == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.server".to_string(),
                reason: "null pointer".to_string(),
            }));
        } else if record.connection == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.connection".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        unsafe {
            let context = match Self::find(&mut *(record.pool)) {
                Some(existing_context) => existing_context,
                None => {
                    let server = &mut *(record.server);
                    let pool = &mut *(record.pool);
                    let connection = &mut *(record.connection);
                    Self::create(record, pool, server, connection)?
                },
            };
            return Ok(context);
        }
    }

    fn find(request_pool: &'r mut apr_pool_t) -> Option<&'r mut Self> {
        let mut context_ptr: *mut RequestContext<'r> = ptr::null_mut();
        unsafe {
            let get_result = apr_pool_userdata_get(
                &mut context_ptr as *mut *mut RequestContext<'r> as *mut *mut c_void,
                RequestContext::USER_DATA_KEY,
                request_pool
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
        record: &'r mut request_rec,
        record_pool: &'r mut apr_pool_t,
        server: &'r mut server_rec,
        connection: &'r mut conn_rec,
    ) -> Result<&'r mut Self, Box<dyn Error>> {
        let pool_ptr = record_pool as *mut apr_pool_t;
        let new_context = alloc::<RequestContext<'r>>(record_pool)?;
        new_context.record = record;
        unsafe {
            apr_pool_userdata_set(
                new_context as *mut _ as *mut c_void,
                RequestContext::USER_DATA_KEY,
                Some(drop_request_context),
                pool_ptr
            );
        }
        new_context.worker = WorkerContext::find_or_create(server)?;
        new_context.connection = ConnectionContext::find_or_create(connection)?;
        return Ok(new_context);
    }
}

#[no_mangle]
pub unsafe extern fn drop_request_context(request_void: *mut c_void) -> apr_status_t {
    if request_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let request_ptr = request_void as *mut RequestContext;
    let request_ref = &mut *request_ptr;
    drop(request_ref);
    return APR_SUCCESS as apr_status_t;
}
