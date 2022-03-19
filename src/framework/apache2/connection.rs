use crate::binding::apache2::{
    apr_status_t, conn_rec, request_rec,
    APR_BADARG, APR_SUCCESS,
};
use crate::schema::apache2::error::InvalidRecordError;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::framework::apache2::record::RequestRecord;

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
