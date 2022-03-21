use crate::binding::apache2::{
    apr_status_t, request_rec,
    APR_BADARG, APR_SUCCESS,
};
use crate::schema::apache2::connection::Connection;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve, PoolStored, };
use crate::framework::apache2::record::{ ConnectionRecord, RequestRecord, };

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_void;


impl<'p> PoolStored<'p> for Connection<'p> {

    fn get_id(request: &request_rec) -> CString {
        let record = request.get_connection_record().unwrap();
        let id = CString::new(format!(
            "{}@-{}",
            type_name::<Self>(),
            record.id,
        )).unwrap();
        id
    }

    fn find_or_allocate_new(request: &'p request_rec) -> Result<&'p mut Connection<'p>, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "Connection::find_or_allocate_new - start");
        let connection_record = request.get_connection_record().unwrap();
        let connection = match retrieve(
            connection_record.get_pool()?,
            &(Self::get_id(request))
        ) {
            Some(existing_connection) => existing_connection,
            None => Self::new(request)?,
        };
        debug!(server_record, "Connection::find_or_allocate_new - finish");
        return Ok(connection);
    }

    fn new(request: &'p request_rec) -> Result<&'p mut Connection<'p>, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "Connection::new - start");
        let connection_record = request.get_connection_record().unwrap();
        let new_connection = alloc::<Connection<'p>>(
            connection_record.get_pool()?,
            &(Self::get_id(request)),
            Some(drop_connection),
        )?.0;
        new_connection.record = connection_record;
        debug!(server_record, "Connection::new - finish");
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
