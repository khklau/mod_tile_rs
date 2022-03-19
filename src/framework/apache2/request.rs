use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS,
    apr_status_t, request_rec,
};
use crate::schema::apache2::error::InvalidRecordError;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve, };
use crate::framework::apache2::record::RequestRecord;

use snowflake::SnowflakeIdGenerator;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::{CStr, CString,};
use std::os::raw::c_void;
use std::ptr;
use std::result::Result;


pub struct Apache2Request<'r> {
    pub record: &'r request_rec,
    pub request_id: i64,
    pub uri: &'r str,
}

impl<'r> Apache2Request<'r> {

    pub fn get_id(record: &request_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    pub fn find_or_create(record: &'r request_rec) -> Result<&'r mut Self, Box<dyn Error>> {
        info!(record.server, "Request::find_or_create - start");
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
            None => Self::create(record)?,
        };
        info!(request.record.server, "Request::find_or_create - finish");
        return Ok(request);
    }

    pub fn create(record: &'r request_rec) -> Result<&'r mut Self, Box<dyn Error>> {
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

        info!(record.get_server_record().unwrap(), "Request::create - start");
        let new_request = alloc::<Apache2Request<'r>>(
            record_pool,
            &(Self::get_id(record)),
            Some(drop_request),
        )?.0;
        new_request.record = record;
        let mut generator = SnowflakeIdGenerator::new(1, 1);
        new_request.request_id = generator.real_time_generate();
        new_request.uri = uri;
        info!(record.get_server_record().unwrap(), "Request::create - finish");
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
    use std::boxed::Box;
    use std::error::Error;

    impl<'r> Apache2Request<'r> {
        pub fn create_with_tile_config(
            record: &'r request_rec,
        ) -> Result<&'r mut Self, Box<dyn Error>> {
            Apache2Request::create(record)
        }
    }

}
