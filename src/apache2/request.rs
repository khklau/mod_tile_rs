use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS,
    ap_set_module_config, apr_pool_userdata_set, apr_status_t, request_rec,
};
use crate::apache2::memory::{
    AllocError, MemoryPool, alloc,
};

use std::ffi::CString;
use std::os::raw::{c_char, c_void,};
use std::result::Result;
use std::ptr;

pub struct Request<'r> {
    record: &'r mut request_rec,
    pool: MemoryPool<'r>,
    file_name: Option<CString>,
}

impl<'r> Request<'r> {
    const USER_DATA_KEY: *const c_char = cstr!(module_path!());

    fn new(record: &'r mut request_rec) -> Result<&'r mut Self, AllocError> {
        if record.pool == ptr::null_mut() {
            return Err(AllocError{})
        }
        unsafe {
            let rec_pool = &mut *(record.pool);
            let request = alloc::<Request<'r>>(rec_pool)?;
            let mem_pool = MemoryPool::new(record.pool)?;
            request.record = record;
            request.pool = mem_pool;
            apr_pool_userdata_set(
                request as *mut _ as *mut c_void,
                Request::USER_DATA_KEY,
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
    let request_ptr = request_void as *mut Request;
    let request_ref = &mut *request_ptr;
    drop(request_ref);
    return APR_SUCCESS as apr_status_t;
}
