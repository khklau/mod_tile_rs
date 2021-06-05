#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    apr_pool_t, apr_status_t, server_rec, apr_pool_userdata_get, apr_pool_userdata_set,
    APR_BADARG, APR_SUCCESS, APLOG_ERR, HTTP_INTERNAL_SERVER_ERROR, OK,
};
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::alloc;

use std::boxed::Box;
use std::cell::RefCell;
use std::error::Error;
use std::fs::{File, OpenOptions,};
use std::io::Write;
use std::option::Option;
use std::os::raw::{c_char, c_int, c_void,};
use std::path::PathBuf;
use std::process;
use std::ptr;

pub struct WorkerContext<'w> {
    record: &'w mut server_rec,
    trace_path: PathBuf,
    trace_file: RefCell<File>,
}

impl<'w> WorkerContext<'w> {
    const USER_DATA_KEY: *const c_char = cstr!(module_path!());

    pub fn find_or_create(record: &'w mut server_rec) -> Result<&'w mut Self, Box<dyn Error>> {
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
            let worker = match Self::find(&mut *(proc_record.pool)) {
                Some(existing_worker) => existing_worker,
                None => Self::create(record, &mut *(proc_record.pool))?,
            };
            return Ok(worker);
        }
    }

    fn find(process_pool: &'w mut apr_pool_t) -> Option<&'w mut Self> {
        let mut worker_ptr: *mut WorkerContext<'w> = ptr::null_mut();
        unsafe {
            let get_result = apr_pool_userdata_get(
                &mut worker_ptr as *mut *mut WorkerContext<'w> as *mut *mut c_void,
                WorkerContext::USER_DATA_KEY,
                process_pool
            );
            if get_result == (APR_SUCCESS as i32) {
                let existing_worker = &mut (*worker_ptr);
                return Some(existing_worker);
            } else {
                return None;
            }
        }
    }

    fn create(
        record: &'w mut server_rec,
        process_pool: &'w mut apr_pool_t
    ) -> Result<&'w mut Self, Box<dyn Error>> {
        let pool_ptr = process_pool as *mut apr_pool_t;
        let new_worker = alloc::<WorkerContext<'w>>(process_pool)?;
        new_worker.record = record;
        unsafe {
            apr_pool_userdata_set(
                new_worker as *mut _ as *mut c_void,
                WorkerContext::USER_DATA_KEY,
                Some(drop_worker),
                pool_ptr
            );
        }

        let path_str = format!("/tmp/mod_tile_rs-trace-{}.txt", process::id());
        new_worker.trace_path = PathBuf::from(path_str.as_str());
        new_worker.trace_file = RefCell::new(OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_worker.trace_path.as_path())?
        );
        return Ok(new_worker);
    }
}

#[no_mangle]
pub unsafe extern fn drop_worker(worker_void: *mut c_void) -> apr_status_t {
    if worker_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let worker_ptr = worker_void as *mut WorkerContext;
    let worker_ref = &mut *worker_ptr;
    drop(worker_ref);
    return APR_SUCCESS as apr_status_t;
}

#[no_mangle]
pub extern fn on_post_config_read(
    config_pool: *mut apr_pool_t,
    logging_pool: *mut apr_pool_t,
    temp_pool: *mut apr_pool_t,
    server_info: *mut server_rec,
) -> c_int {
    if config_pool != ptr::null_mut()
        && logging_pool != ptr::null_mut()
        && temp_pool != ptr::null_mut()
        && server_info != ptr::null_mut() {
        unsafe {
            let mut context = match WorkerContext::find_or_create(&mut *server_info) {
                Ok(context) => context,
                Err(why) => {
                    log!(APLOG_ERR, server_info, format!("Failed to create WorkerContext: {}", why));
                    return HTTP_INTERNAL_SERVER_ERROR as c_int;
                },
            };
            match _on_post_config_read(&mut context) {
                Ok(_) => return OK as c_int,
                Err(why) => {
                    log!(APLOG_ERR, server_info, format!("Post config read processing failed: {}", why));
                    return HTTP_INTERNAL_SERVER_ERROR as c_int;
                },
            }
        }
    }
    else {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    }
}

fn _on_post_config_read(
    context: &WorkerContext,
) -> Result<(), std::io::Error> {
    context.trace_file.borrow_mut().write_all(b"apache2::worker::on_post_config_read - start\n")?;
    context.trace_file.borrow_mut().write_all(b"apache2::worker::on_post_config_read - finish\n")?;
    Ok(())
}
