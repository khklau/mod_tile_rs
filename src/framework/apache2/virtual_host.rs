use crate::binding::apache2::{
    apr_status_t, request_rec, server_rec,
    APR_BADARG, APR_SUCCESS,
};
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve, PoolStored, };
use crate::framework::apache2::record::{
    ProcessRecord, RequestRecord, ServerRecord,
};

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_void;


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
        let server_record = request.get_server_record()?;
        info!(server_record, "VirtualHost::find_or_allocate_new - start");
        let proc_record = server_rec::get_process_record(server_record.process)?;
        let host = match retrieve(
            proc_record.get_pool(),
            &(Self::get_id(request))
        ) {
            Some(existing_host) => {
                info!(server_record, "VirtualHost::find_or_allocate_new - existing found");
                existing_host
            },
            None => {
                info!(server_record, "VirtualHost::find_or_allocate_new - not found");
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
