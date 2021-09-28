#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    apr_status_t, conn_rec,
    APR_BADARG, APR_SUCCESS,
};
use crate::apache2::error::InvalidArgError;
use crate::apache2::memory::{ alloc, retrieve, };
use crate::apache2::virtual_host::VirtualHostContext;
use crate::tile::config::TileConfig;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

pub struct ConnectionContext<'c> {
    pub record: &'c mut conn_rec,
    pub host: &'c mut VirtualHostContext<'c>,
}

impl<'c> ConnectionContext<'c> {

    pub fn get_id(record: &conn_rec) -> CString {
        let id = CString::new(format!(
            "{}@-{}",
            type_name::<Self>(),
            record.id,
        )).unwrap();
        id
    }

    pub fn find_or_create(
        record: &'c mut conn_rec,
        config: &'c TileConfig,
    ) -> Result<&'c mut Self, Box<dyn Error>> {
        info!(record.base_server, "ConnectionContext::find_or_create - start");
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "conn_rec.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        } else if record.base_server == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "conn_rec.base_server".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        let context = match retrieve(
            unsafe { &mut *(record.pool) },
            &(Self::get_id(record))
        ) {
            Some(existing_context) => existing_context,
            None => {
                let server = unsafe { &mut *(record.base_server) };
                let host_context = VirtualHostContext::find_or_create(server, config)?;
                Self::create(record, host_context)?
            },
        };
        info!(context.record.base_server, "ConnectionContext::find_or_create - finish");
        return Ok(context);
    }

    pub fn create(
        record: &'c mut conn_rec,
        host_context: &'c mut VirtualHostContext<'c>,
    ) -> Result<&'c mut Self, Box<dyn Error>> {
        info!(record.base_server, "ConnectionContext::create - start");
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "conn_rec.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        let new_context = alloc::<ConnectionContext<'c>>(
            unsafe { &mut *(record.pool) },
            &(Self::get_id(record)),
            Some(drop_connection_context),
        )?.0;
        new_context.record = record;
        new_context.host = host_context;
        info!(new_context.record.base_server, "ConnectionContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
pub unsafe extern fn drop_connection_context(connection_void: *mut c_void) -> apr_status_t {
    if connection_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let context_ptr = connection_void as *mut ConnectionContext;
    info!((&mut *context_ptr).record.base_server, "drop_connection_context - start");
    let context_ref = &mut *context_ptr;
    drop(context_ref);
    info!((&mut *context_ptr).record.base_server, "drop_connection_context - finish");
    return APR_SUCCESS as apr_status_t;
}

#[cfg(test)]
pub mod test_utils {
    use super::ConnectionContext;
    use crate::apache2::bindings::{
        __BindgenBitfieldUnit, ap_conn_keepalive_e, apr_pool_t, conn_rec, server_rec,
    };
    use crate::apache2::memory::test_utils::with_pool;
    use crate::apache2::virtual_host::VirtualHostContext;
    use crate::apache2::virtual_host::test_utils::with_server_rec;
    use crate::tile::config::TileConfig;
    use std::boxed::Box;
    use std::error::Error;
    use std::ops::FnOnce;
    use std::os::raw::{ c_int, c_long, c_uint, };
    use std::ptr;

    impl<'c> ConnectionContext<'c> {
        pub fn create_with_tile_config(
            record: &'c mut conn_rec,
            config: &'c TileConfig,
        ) -> Result<&'c mut Self, Box<dyn Error>> {
            let server = unsafe { &mut *(record.base_server) };
            let host_context = VirtualHostContext::create(server, config)?;
            ConnectionContext::create(record, host_context)
        }
    }

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
