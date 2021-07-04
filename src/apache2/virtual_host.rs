#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    apr_pool_t, apr_status_t, server_rec, apr_pool_userdata_get, apr_pool_userdata_set,
    APR_BADARG, APR_SUCCESS, APLOG_ERR,
};
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::alloc;

use std::any::type_name;
use std::boxed::Box;
use std::cell::RefCell;
use std::error::Error;
use std::ffi::CString;
use std::fs::{File, OpenOptions,};
use std::option::Option;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::process;
use std::ptr;
use thread_id;

pub struct VirtualHostContext<'h> {
    pub record: &'h mut server_rec,
    pub trace_path: PathBuf,
    pub trace_file: RefCell<File>,
}

impl<'h> VirtualHostContext<'h> {

    pub fn get_id() -> CString {
        let id = CString::new(format!(
            "{}-pid{}-tid{}",
            type_name::<Self>(),
            process::id(),
            thread_id::get(),
        )).unwrap();
        id
    }

    pub fn find_or_create(record: &'h mut server_rec) -> Result<&'h mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, record, "VirtualHostContext::find_or_create - start");
        if record.process == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "server_rec.process".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        unsafe {
            let proc_record = &mut *(record.process);
            if proc_record.pool == ptr::null_mut() {
                return Err(Box::new(InvalidArgError{
                    arg: "server_rec.process.pool".to_string(),
                    reason: "null pointer".to_string(),
                }));
            }
            let context = match Self::find(&mut *(proc_record.pool)) {
                Some(existing_context) => {
                    log!(APLOG_ERR, record, "VirtualHostContext::find_or_create - existing found");
                    existing_context
                },
                None => {
                    log!(APLOG_ERR, record, "VirtualHostContext::find_or_create - not found");
                    Self::create(record, &mut *(proc_record.pool))?
                },
            };
            log!(APLOG_ERR, context.record, "VirtualHostContext::find_or_create - finish");
            return Ok(context);
        }
    }

    fn find(process_pool: &'h mut apr_pool_t) -> Option<&'h mut Self> {
        plog!(APLOG_ERR, process_pool, "VirtualHostContext::find searching - start");
        let mut context_ptr: *mut VirtualHostContext<'h> = ptr::null_mut();
        let user_data_key = VirtualHostContext::get_id();
        unsafe {
            let get_result = apr_pool_userdata_get(
                &mut context_ptr as *mut *mut VirtualHostContext<'h> as *mut *mut c_void,
                user_data_key.as_ptr(),
                process_pool
            );
            if get_result == (APR_SUCCESS as i32) {
                let existing_context = &mut (*context_ptr);
                plog!(APLOG_ERR, process_pool, "VirtualHostContext::find success - finish");
                return Some(existing_context);
            } else {
                plog!(APLOG_ERR, process_pool, "VirtualHostContext::find failed - finish");
                return None;
            }
        }
    }

    fn create(
        record: &'h mut server_rec,
        process_pool: &'h mut apr_pool_t
    ) -> Result<&'h mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, record, "VirtualHostContext::create - start");
        let pool_ptr = process_pool as *mut apr_pool_t;
        let new_context = alloc::<VirtualHostContext<'h>>(process_pool)?;
        new_context.record = record;

        let path_str = format!(
            "/tmp/mod_tile_rs-trace-pid{}-tid{}.txt",
            process::id(),
            thread_id::get(),
        );
        new_context.trace_path = PathBuf::from(path_str.as_str());
        new_context.trace_file = RefCell::new(OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_context.trace_path.as_path())?
        );

        let user_data_key = VirtualHostContext::get_id();
        unsafe {
            apr_pool_userdata_set(
                new_context as *mut _ as *mut c_void,
                user_data_key.as_ptr(),
                Some(drop_virtual_host_context),
                pool_ptr,
            );
        }
        log!(APLOG_ERR, new_context.record, "VirtualHostContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
pub unsafe extern fn drop_virtual_host_context(host_void: *mut c_void) -> apr_status_t {
    if host_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let context_ptr = host_void as *mut VirtualHostContext;
    log!(APLOG_ERR, (&mut *context_ptr).record, "drop_virtual_host_context - start");
    let context_ref = &mut *context_ptr;
    drop(context_ref);
    log!(APLOG_ERR, (&mut *context_ptr).record, "drop_virtual_host_context - finish");
    return APR_SUCCESS as apr_status_t;
}
