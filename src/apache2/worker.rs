#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    apr_pool_t, apr_status_t, server_rec, apr_pool_userdata_get, apr_pool_userdata_set,
    APR_BADARG, APR_SUCCESS, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::alloc;

use std::any::type_name;
use std::boxed::Box;
use std::cell::RefCell;
use std::error::Error;
use std::ffi::CString;
use std::fs::{File, OpenOptions,};
use std::io::Write;
use std::option::Option;
use std::os::raw::{c_int, c_void,};
use std::path::PathBuf;
use std::process;
use std::ptr;

pub struct WorkerContext<'w> {
    pub record: &'w mut server_rec,
    pub trace_path: PathBuf,
    pub trace_file: RefCell<File>,
}

impl<'w> WorkerContext<'w> {

    pub fn get_id() -> CString {
        let id = CString::new(format!(
            "{}-pid{}-tid{}",
            type_name::<Self>(),
            process::id(),
            // TODO : use thread_id instead
            unsafe {
                libc::pthread_self()
            },
        )).unwrap();
        id
    }

    pub fn find_or_create(record: &'w mut server_rec) -> Result<&'w mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, record, "WorkerContext::find_or_create - start");
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
                    log!(APLOG_ERR, record, "WorkerContext::find_or_create - existing found");
                    existing_context
                },
                None => {
                    log!(APLOG_ERR, record, "WorkerContext::find_or_create - not found");
                    Self::create(record, &mut *(proc_record.pool))?
                },
            };
            log!(APLOG_ERR, context.record, "WorkerContext::find_or_create - finish");
            return Ok(context);
        }
    }

    fn find(process_pool: &'w mut apr_pool_t) -> Option<&'w mut Self> {
        plog!(APLOG_ERR, process_pool, "WorkerContext::find searching - start");
        let mut context_ptr: *mut WorkerContext<'w> = ptr::null_mut();
        let user_data_key = WorkerContext::get_id();
        unsafe {
            let get_result = apr_pool_userdata_get(
                &mut context_ptr as *mut *mut WorkerContext<'w> as *mut *mut c_void,
                user_data_key.as_ptr(),
                process_pool
            );
            if get_result == (APR_SUCCESS as i32) {
                let existing_context = &mut (*context_ptr);
                plog!(APLOG_ERR, process_pool, "WorkerContext::find success - finish");
                return Some(existing_context);
            } else {
                plog!(APLOG_ERR, process_pool, "WorkerContext::find failed - finish");
                return None;
            }
        }
    }

    fn create(
        record: &'w mut server_rec,
        process_pool: &'w mut apr_pool_t
    ) -> Result<&'w mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, record, "WorkerContext::create - start");
        let pool_ptr = process_pool as *mut apr_pool_t;
        let new_context = alloc::<WorkerContext<'w>>(process_pool)?;
        new_context.record = record;

        let path_str = format!(
            "/tmp/mod_tile_rs-trace-pid{}-tid{}.txt",
            process::id(),
            // TODO : use thread_id instead
            unsafe {
                libc::pthread_self()
            },
        );
        new_context.trace_path = PathBuf::from(path_str.as_str());
        new_context.trace_file = RefCell::new(OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_context.trace_path.as_path())?
        );

        let user_data_key = WorkerContext::get_id();
        unsafe {
            apr_pool_userdata_set(
                new_context as *mut _ as *mut c_void,
                user_data_key.as_ptr(),
                Some(drop_worker_context),
                pool_ptr,
            );
        }
        log!(APLOG_ERR, new_context.record, "WorkerContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
pub unsafe extern fn drop_worker_context(worker_void: *mut c_void) -> apr_status_t {
    if worker_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let context_ptr = worker_void as *mut WorkerContext;
    log!(APLOG_ERR, (&mut *context_ptr).record, "drop_worker_context - start");
    let context_ref = &mut *context_ptr;
    drop(context_ref);
    log!(APLOG_ERR, (&mut *context_ptr).record, "drop_worker_context - finish");
    return APR_SUCCESS as apr_status_t;
}
