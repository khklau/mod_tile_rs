use crate::binding::apache2::{
    apr_status_t, conn_rec, request_rec,
    APR_BADARG, APR_SUCCESS,
};
use crate::schema::apache2::error::InvalidRecordError;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::apache2::request::RequestRecord;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;


pub struct Connection<'c> {
    pub record: &'c conn_rec,
}

impl<'c> Connection<'c> {

    pub fn get_id(record: &conn_rec) -> CString {
        let id = CString::new(format!(
            "{}@-{}",
            type_name::<Self>(),
            record.id,
        )).unwrap();
        id
    }

    pub fn find_or_make_new(request: &'c request_rec) -> Result<&'c mut Self, Box<dyn Error>> {
        let record = request.get_connection_record().unwrap();
        info!(record.base_server, "Connection::find_or_create - start");
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                record as *const conn_rec,
                "pool field is null pointer",
            )));
        } else if record.base_server == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                record as *const conn_rec,
                "base_server field is null pointer",
            )));
        }
        let connection = match retrieve(
            unsafe { record.pool.as_mut().unwrap() },
            &(Self::get_id(record))
        ) {
            Some(existing_connection) => existing_connection,
            None => Self::new(request)?,
        };
        info!(connection.record.base_server, "Connection::find_or_create - finish");
        return Ok(connection);
    }

    pub fn new(request: &'c request_rec) -> Result<&'c mut Self, Box<dyn Error>> {
        let record = request.get_connection_record().unwrap();
        info!(record.base_server, "Connection::create - start");
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                record as *const conn_rec,
                "pool field is null pointer",
            )));
        }
        let new_connection = alloc::<Connection<'c>>(
            unsafe { record.pool.as_mut().unwrap() },
            &(Self::get_id(record)),
            Some(drop_connection),
        )?.0;
        new_connection.record = record;
        info!(new_connection.record.base_server, "Connection::create - finish");
        return Ok(new_connection);
    }
}

#[no_mangle]
extern "C" fn drop_connection(connection_void: *mut c_void) -> apr_status_t {
    let connection_ref = match access_pool_object::<Connection>(connection_void) {
        None => {
            return APR_BADARG as apr_status_t;
        },
        Some(host) => host,
    };
    info!(connection_ref.record.base_server, "drop_connection - dropping");
    drop(connection_ref);
    return APR_SUCCESS as apr_status_t;
}

#[cfg(test)]
pub mod test_utils {
    use crate::binding::apache2::{
        __BindgenBitfieldUnit, ap_conn_keepalive_e, apr_pool_t, conn_rec, server_rec,
    };
    use crate::framework::apache2::memory::test_utils::with_pool;
    use crate::apache2::virtual_host::test_utils::with_server_rec;
    use std::boxed::Box;
    use std::error::Error;
    use std::ops::FnOnce;
    use std::os::raw::{ c_int, c_long, c_uint, };
    use std::ptr;

    impl conn_rec {
        pub fn new() -> conn_rec {
            conn_rec {
                pool: ptr::null_mut(),
                base_server: ptr::null_mut(),
                vhost_lookup_data: ptr::null_mut(),
                local_addr: ptr::null_mut(),
                client_addr: ptr::null_mut(),
                client_ip: ptr::null_mut(),
                remote_host: ptr::null_mut(),
                remote_logname: ptr::null_mut(),
                local_ip: ptr::null_mut(),
                local_host: ptr::null_mut(),
                id: 0 as c_long,
                conn_config: ptr::null_mut(),
                notes: ptr::null_mut(),
                input_filters: ptr::null_mut(),
                output_filters: ptr::null_mut(),
                sbh: ptr::null_mut(),
                bucket_alloc: ptr::null_mut(),
                cs: ptr::null_mut(),
                data_in_input_filters: 0 as c_int,
                data_in_output_filters: 0 as c_int,
                _bitfield_align_1: [],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 1usize]),
                aborted: 0 as c_uint,
                keepalive: 0 as ap_conn_keepalive_e,
                keepalives: 0 as c_int,
                log: ptr::null(),
                log_id: ptr::null(),
                current_thread: ptr::null_mut(),
                master: ptr::null_mut(),
            }
        }
    }

    pub fn with_conn_rec<F>(func: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce(&mut conn_rec) -> Result<(), Box<dyn Error>> {
        let mut record: conn_rec = conn_rec::new();
        with_pool(|pool| {
            record.pool = pool as *mut apr_pool_t;
            with_server_rec(|server| {
                record.base_server = server as *mut server_rec;
                func(&mut record)
            })
        })
    }
}
