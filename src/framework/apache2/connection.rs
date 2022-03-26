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

    fn search_pool_key(request: &request_rec) -> CString {
        let record = request.get_connection_record().unwrap();
        CString::new(format!(
            "{}@-{}",
            type_name::<Self>(),
            record.id,
        )).unwrap()
    }

    fn find(
        request: &'p request_rec,
        pool_key: &CString,
    ) -> Result<Option<&'p mut Connection<'p>>, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "Connection::find - start");
        let connection_record = request.get_connection_record().unwrap();
        let pool = connection_record.get_pool()?;
        let existing_connection = retrieve(pool, pool_key);
        debug!(server_record, "Connection::find - finish");
        Ok(existing_connection)
    }

    fn new(request: &'p request_rec) -> Result<&'p mut Connection<'p>, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "Connection::new - start");
        let connection_record = request.get_connection_record().unwrap();
        let pool = connection_record.get_pool()?;
        let key = Self::search_pool_key(request);
        let new_connection = alloc::<Connection<'p>>(
            pool,
            &key,
            Some(drop_connection),
        )?.0;
        new_connection.record = connection_record;
        new_connection.connection_id = connection_record.id;
        debug!(server_record, "Connection::new - finish");
        Ok(new_connection)
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
