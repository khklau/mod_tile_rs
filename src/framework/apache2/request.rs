use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS,
    apr_status_t, request_rec,
};
use crate::schema::apache2::error::InvalidRecordError;
use crate::schema::apache2::request::Apache2Request;
use crate::core::memory::PoolStored;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve, };
use crate::framework::apache2::record::RequestRecord;

use chrono::{TimeZone, Utc,};
use snowflake::SnowflakeIdGenerator;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::{CStr, CString,};
use std::os::raw::c_void;
use std::ptr;
use std::result::Result;


impl<'p> PoolStored<'p> for Apache2Request<'p> {

    fn search_pool_key(record: &request_rec) -> CString {
        CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap()
    }

    fn find(
        request: &'p request_rec,
        pool_key: &CString,
    ) -> Result<Option<&'p mut Apache2Request<'p>>, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "Apache2Request::find - start");
        let pool = request.get_pool()?;
        let existing_request = retrieve(pool, pool_key);
        debug!(server_record, "Apache2Request::find - finish");
        Ok(existing_request)
    }

    fn new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "Apache2Request::new - start");
        if request.uri == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                request as *const request_rec,
                "uri field is null pointer",
            )));
        }
        let uri = unsafe { CStr::from_ptr(request.uri).to_str()? };
        let pool = request.get_pool()?;
        let key = Self::search_pool_key(request);
        let new_request = alloc::<Apache2Request<'p>>(
            pool,
            &key,
            Some(drop_request),
        )?.0;
        new_request.record = request;
        let mut generator = SnowflakeIdGenerator::new(1, 1);
        new_request.request_id = generator.real_time_generate();
        new_request.uri = uri;
        new_request.received_timestamp = Utc.timestamp_millis(request.request_time);
        debug!(server_record, "Apache2Request::new - finish");
        Ok(new_request)
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
    use crate::core::memory::PoolStored;
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
