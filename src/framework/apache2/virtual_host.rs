use crate::binding::apache2::{
    apr_status_t, request_rec,
    APR_BADARG, APR_SUCCESS,
};
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve, PoolStored, };
use crate::framework::apache2::record::{ RequestRecord, ServerRecord, };

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_void;
use std::option::Option;


impl<'p> PoolStored<'p> for VirtualHost<'p> {

    fn search_pool_key(request: &request_rec) -> CString {
        let record = request.get_server_record().unwrap();
        CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap()
    }

    fn find(
        request: &'p request_rec,
        pool_key: &CString,
    ) -> Result<Option<&'p mut VirtualHost<'p>>, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "VirtualHost::find - start");
        let pool = server_record.get_pool()?;
        let existing_host = retrieve(pool, pool_key);
        debug!(server_record, "VirtualHost::find - finish");
        Ok(existing_host)
    }

    fn new(request: &'p request_rec) -> Result<&'p mut VirtualHost<'p>, Box<dyn Error>> {
        let server_record = request.get_server_record()?;
        debug!(server_record, "VirtualHost::new - start");
        let pool = server_record.get_pool()?;
        let key = Self::search_pool_key(request);
        let new_host = alloc::<VirtualHost<'p>>(
            pool,
            &key,
            Some(drop_virtual_host),
        )?.0;
        new_host.record = request.get_server_record()?;
        debug!(server_record, "VirtualHost::new - finish");
        Ok(new_host)
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
