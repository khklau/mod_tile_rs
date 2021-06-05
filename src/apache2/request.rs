use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS,
    ap_set_module_config, apr_pool_userdata_set, apr_status_t, request_rec,
};
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::alloc;

use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::{c_char, c_void,};
use std::result::Result;
use std::ptr;

pub struct RequestContext<'r> {
    pub record: &'r mut request_rec,
    pub file_name: Option<CString>,
}

impl<'r> RequestContext<'r> {
    const USER_DATA_KEY: *const c_char = cstr!(module_path!());

    pub fn new(record: &'r mut request_rec) -> Result<&'r mut Self, Box<dyn Error>> {
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        unsafe {
            let rec_pool = &mut *(record.pool);
            let request = alloc::<RequestContext<'r>>(rec_pool)?;
            request.record = record;
            apr_pool_userdata_set(
                request as *mut _ as *mut c_void,
                RequestContext::USER_DATA_KEY,
                Some(drop_request),
                request.record.pool
            );
            ap_set_module_config(
                request.record.request_config,
                &crate::TILE_MODULE,
                request as *mut _ as *mut c_void);
            return Ok(request);
        }
    }
}

#[no_mangle]
pub unsafe extern fn drop_request(request_void: *mut c_void) -> apr_status_t {
    if request_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let request_ptr = request_void as *mut RequestContext;
    let request_ref = &mut *request_ptr;
    drop(request_ref);
    return APR_SUCCESS as apr_status_t;
}