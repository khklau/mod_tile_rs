use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS,
    apr_status_t, request_rec,
};
use crate::schema::apache2::error::InvalidRecordError;
use crate::schema::apache2::request::Apache2Request;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve, PoolStored, };
use crate::framework::apache2::record::RequestRecord;

use snowflake::SnowflakeIdGenerator;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::{CStr, CString,};
use std::os::raw::c_void;
use std::ptr;
use std::result::Result;


impl<'p> PoolStored<'p> for Apache2Request<'p> {

    fn get_id(record: &request_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    fn find_or_allocate_new(record: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>> {
        debug!(record.server, "Apache2Request::find_or_allocate_new - start");
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                record as *const request_rec,
                "pool field is null pointer",
            )));
        } else if record.connection == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                record as *const request_rec,
                "connection field is null pointer",
            )));
        }
        let request = match retrieve(
            unsafe { record.pool.as_mut().unwrap() },
            &(Self::get_id(record))
        ) {
            Some(existing_request) => existing_request,
            None => Self::new(record)?,
        };
        debug!(request.record.server, "Apache2Request::find_or_allocate_new - finish");
        return Ok(request);
    }

    fn new(record: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>> {
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                record as *const request_rec,
                "pool field is null pointer",
            )));
        } else if record.uri == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                record as *const request_rec,
                "uri field is null pointer",
            )));
        }
        let record_pool = unsafe { record.pool.as_mut().unwrap() };
        let uri = unsafe { CStr::from_ptr(record.uri).to_str()? };

        info!(record.get_server_record().unwrap(), "Request::new - start");
        let new_request = alloc::<Apache2Request<'p>>(
            record_pool,
            &(Self::get_id(record)),
            Some(drop_request),
        )?.0;
        new_request.record = record;
        let mut generator = SnowflakeIdGenerator::new(1, 1);
        new_request.request_id = generator.real_time_generate();
        new_request.uri = uri;
        info!(record.get_server_record().unwrap(), "Request::new - finish");
        return Ok(new_request);
    }
}

#[no_mangle]
extern "C" fn drop_request(request_void: *mut c_void) -> apr_status_t {
    let request_ref = match access_pool_object::<Apache2Request>(request_void) {
        None => {
            return APR_BADARG as apr_status_t;
        },
        Some(host) => host,
    };
    info!(request_ref.record.server, "drop_connection - dropping");
    drop(request_ref);
    return APR_SUCCESS as apr_status_t;
}

#[cfg(test)]
pub mod test_utils {
    use super::Apache2Request;
    use crate::binding::apache2::request_rec;
    use crate::framework::apache2::memory::PoolStored;
    use std::boxed::Box;
    use std::error::Error;

    impl<'p> Apache2Request<'p> {
        pub fn create_with_tile_config(
            record: &'p request_rec,
        ) -> Result<&'p mut Self, Box<dyn Error>> {
            Apache2Request::new(record)
        }
    }

}
