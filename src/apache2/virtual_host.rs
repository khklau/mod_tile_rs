use crate::apache2::memory::{ access_pool_object, alloc, retrieve };

use crate::binding::apache2::{
    apr_pool_t, apr_status_t, process_rec, server_rec,
    APR_BADARG, APR_SUCCESS,
};
use crate::schema::apache2::error::InvalidRecordError;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::{ CStr, CString, };
use std::option::Option;
use std::os::raw::c_void;
use std::ptr;


pub trait ServerRecord {
    fn get_host_name<'s>(&'s self) -> Option<&'s str>;

    fn get_process_record<'s>(process: *mut process_rec) -> Result<&'s process_rec, InvalidRecordError>;
}

impl ServerRecord for server_rec {
    fn get_host_name<'s>(&'s self) -> Option<&'s str> {
        if self.server_hostname == ptr::null_mut() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.server_hostname).to_str().unwrap() })
        }
    }

    fn get_process_record<'s>(process: *mut process_rec) -> Result<&'s process_rec, InvalidRecordError> {
        if process == ptr::null_mut() {
            return Err(InvalidRecordError::new(
                process,
                "server_rec.process field is a null pointer",
            ));
        }
        let proc_record = unsafe { &mut *(process) };
        if proc_record.pool == ptr::null_mut() {
            return Err(InvalidRecordError::new(
                proc_record as *const process_rec,
                "server_rec.process.pool field is a null pointer",
            ));
        }
        Ok(proc_record)
    }
}

pub trait ProcessRecord {
    fn get_pool<'p>(&'p self) -> &'p mut apr_pool_t;
}

impl ProcessRecord for process_rec {
    fn get_pool<'p>(&'p self) -> &'p mut apr_pool_t {
        unsafe { &mut *(self.pool) }
    }
}


pub struct VirtualHost<'h> {
    pub record: &'h mut server_rec,
}

impl<'h> VirtualHost<'h> {

    pub fn get_id(record: &server_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    pub fn find_or_create(record: &'h mut server_rec) -> Result<&'h mut Self, Box<dyn Error>> {
        info!(record, "VirtualHostContext::find_or_create - start");
        let proc_record = server_rec::get_process_record(record.process)?;
        let context = match retrieve(
            proc_record.get_pool(),
            &(Self::get_id(record))
        ) {
            Some(existing_context) => {
                info!(record, "VirtualHostContext::find_or_create - existing found");
                existing_context
            },
            None => {
                info!(record, "VirtualHostContext::find_or_create - not found");
                Self::create(record)?
            },
        };
        info!(context.record, "VirtualHostContext::find_or_create - finish");
        return Ok(context);
    }

    pub fn create(record: &'h mut server_rec) -> Result<&'h mut Self, Box<dyn Error>> {
        info!(record, "VirtualHostContext::create - start");
        let proc_record = server_rec::get_process_record(record.process)?;
        let new_context = alloc::<VirtualHost<'h>>(
            proc_record.get_pool(),
            &(Self::get_id(record)),
            Some(drop_virtual_host_context),
        )?.0;
        new_context.record = record;
        info!(new_context.record, "VirtualHostContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
extern "C" fn drop_virtual_host_context(context_void: *mut c_void) -> apr_status_t {
    let context_ref = match access_pool_object::<VirtualHost>(context_void) {
        None => {
            return APR_BADARG as apr_status_t;
        },
        Some(host) => host,
    };
    info!(context_ref.record, "drop_virtual_host_context - dropping");
    drop(context_ref);
    return APR_SUCCESS as apr_status_t;
}

#[cfg(test)]
pub mod test_utils {
    use crate::binding::apache2::{
        __BindgenBitfieldUnit, ap_logconf,
        apr_interval_time_t, apr_pool_t, apr_port_t,
        process_rec, server_rec,
    };
    use crate::apache2::memory::test_utils::with_pool;
    use std::boxed::Box;
    use std::error::Error;
    use std::ops::FnOnce;
    use std::os::raw::{ c_char, c_int, };
    use std::ptr;

    impl ap_logconf {
        pub fn new() -> ap_logconf {
            ap_logconf {
                module_levels: ptr::null_mut(),
                level: 0,
            }
        }
    }

    impl server_rec {
        pub fn new() -> server_rec {
            server_rec {
                process: ptr::null_mut(),
                next: ptr::null_mut(),
                error_fname: ptr::null_mut(),
                error_log: ptr::null_mut(),
                log: ap_logconf::new(),
                module_config: ptr::null_mut(),
                lookup_defaults: ptr::null_mut(),
                defn_name: ptr::null(),
                defn_line_number: 0,
                is_virtual: '\0' as c_char,
                port: 0 as apr_port_t,
                server_scheme: ptr::null(),
                server_admin: ptr::null_mut(),
                server_hostname: cstr!("localhost") as *mut i8,
                addrs: ptr::null_mut(),
                timeout: 0 as apr_interval_time_t,
                keep_alive_timeout: 0 as apr_interval_time_t,
                keep_alive_max: 0 as c_int,
                keep_alive: 0 as c_int,
                names: ptr::null_mut(),
                wild_names: ptr::null_mut(),
                path: ptr::null(),
                pathlen: 0 as c_int,
                limit_req_line: 0 as c_int,
                limit_req_fieldsize: 0 as c_int,
                limit_req_fields: 0 as c_int,
                context: ptr::null_mut(),
                _bitfield_align_1: [],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 1usize]),
                __bindgen_padding_0: [0; 7usize],
            }
        }
    }

    pub fn with_server_rec<F>(func: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce(&mut server_rec) -> Result<(), Box<dyn Error>> {
        let mut process: process_rec = process_rec {
            pool: ptr::null_mut(),
            pconf: ptr::null_mut(),
            short_name: cstr!("test"),
            argv: ptr::null_mut(),
            argc: 0,
        };
        let mut record: server_rec = server_rec::new();
        record.process = &mut process;
        with_pool(|proc_pool| {
            process.pool = proc_pool as *mut apr_pool_t;
            with_pool(|conf_pool| {
                process.pconf = conf_pool as *mut apr_pool_t;
                func(&mut record)
            })
        })
    }
}
