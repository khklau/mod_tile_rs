use crate::schema::apache2::error::InvalidRecordError;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::interface::apache2::pool::PoolStored;
use crate::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::apache2::request::RequestRecord;

use crate::binding::apache2::{
    apr_pool_t, apr_status_t, process_rec, request_rec, server_rec,
    APR_BADARG, APR_SUCCESS,
};

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
        let proc_record = unsafe { process.as_mut().unwrap() };
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
        unsafe { self.pool.as_mut().unwrap() }
    }
}


impl<'p> PoolStored<'p> for VirtualHost<'p> {

    fn get_id(request: &request_rec) -> CString {
        let record = request.get_server_record().unwrap();
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    fn find_or_allocate_new(request: &'p request_rec) -> Result<&'p mut VirtualHost<'p>, Box<dyn Error>> {
        let record = request.get_server_record()?;
        info!(record, "VirtualHost::find_or_allocate_new - start");
        let proc_record = server_rec::get_process_record(record.process)?;
        let host = match retrieve(
            proc_record.get_pool(),
            &(Self::get_id(request))
        ) {
            Some(existing_host) => {
                info!(record, "VirtualHost::find_or_allocate_new - existing found");
                existing_host
            },
            None => {
                info!(record, "VirtualHost::find_or_allocate_new - not found");
                Self::new(request)?
            },
        };
        info!(host.record, "VirtualHost::find_or_allocate_new - finish");
        return Ok(host);
    }

    fn new(request: &'p request_rec) -> Result<&'p mut VirtualHost<'p>, Box<dyn Error>> {
        debug!(request.get_server_record()?, "VirtualHost::new - start");
        let proc_record = server_rec::get_process_record(request.get_server_record()?.process)?;
        let new_host = alloc::<VirtualHost<'p>>(
            proc_record.get_pool(),
            &(Self::get_id(request)),
            Some(drop_virtual_host),
        )?.0;
        new_host.record = request.get_server_record()?;
        debug!(new_host.record, "VirtualHost::new - finish");
        return Ok(new_host);
    }
}

#[no_mangle]
extern "C" fn drop_virtual_host(host_void: *mut c_void) -> apr_status_t {
    let host_ref = match access_pool_object::<VirtualHost>(host_void) {
        None => {
            return APR_BADARG as apr_status_t;
        },
        Some(host) => host,
    };
    info!(host_ref.record, "drop_virtual_host - dropping");
    drop(host_ref);
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
