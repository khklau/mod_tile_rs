use crate::apache2::connection::ConnectionContext;
use crate::apache2::memory::{ access_pool_object, alloc, retrieve, };
use crate::apache2::virtual_host::VirtualHostContext;

use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS,
    apr_status_t, request_rec,
};
use crate::schema::apache2::error::InvalidRecordError;
use crate::schema::tile::config::ModuleConfig;

use snowflake::SnowflakeIdGenerator;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::{CStr, CString,};
use std::os::raw::c_void;
use std::ptr;
use std::result::Result;


pub struct RequestContext<'r> {
    pub record: &'r mut request_rec,
    pub connection: &'r mut ConnectionContext<'r>,
    pub request_id: i64,
    pub uri: &'r str,
}

impl<'r> RequestContext<'r> {

    pub fn get_context_id(record: &request_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    pub fn get_host(&self) -> &VirtualHostContext {
        self.connection.host
    }

    pub fn get_config(&self) -> &ModuleConfig{
        self.get_host().tile_config
    }

    pub fn find_or_create(
        record: &'r mut request_rec,
        config: &'r ModuleConfig,
    ) -> Result<&'r mut Self, Box<dyn Error>> {
        info!(record.server, "RequestContext::find_or_create - start");
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
        let context = match retrieve(
            unsafe { &mut *(record.pool) },
            &(Self::get_context_id(record))
        ) {
            Some(existing_context) => existing_context,
            None => {
                let connection = unsafe { &mut *(record.connection) };
                let conn_context = ConnectionContext::find_or_create(connection, config)?;
                Self::create(record, conn_context)?
            },
        };
        info!(context.record.server, "RequestContext::find_or_create - finish");
        return Ok(context);
    }

    pub fn create(
        record: &'r mut request_rec,
        conn_context: &'r mut ConnectionContext<'r>,
    ) -> Result<&'r mut Self, Box<dyn Error>> {
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
        let record_pool = unsafe { &mut *(record.pool) };
        let uri = unsafe { CStr::from_ptr(record.uri).to_str()? };

        info!(conn_context.host.record, "RequestContext::create - start");
        let new_context = alloc::<RequestContext<'r>>(
            record_pool,
            &(Self::get_context_id(record)),
            Some(drop_request_context),
        )?.0;
        new_context.record = record;
        new_context.connection = conn_context;
        let mut generator = SnowflakeIdGenerator::new(1, 1);
        new_context.request_id = generator.real_time_generate();
        new_context.uri = uri;
        info!(new_context.connection.host.record, "RequestContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
extern "C" fn drop_request_context(context_void: *mut c_void) -> apr_status_t {
    let context_ref = match access_pool_object::<RequestContext>(context_void) {
        None => {
            return APR_BADARG as apr_status_t;
        },
        Some(host) => host,
    };
    info!(context_ref.record.server, "drop_connection_context - dropping");
    drop(context_ref);
    return APR_SUCCESS as apr_status_t;
}

#[cfg(test)]
pub mod test_utils {
    use super::RequestContext;
    use crate::binding::apache2::{
        __BindgenBitfieldUnit, apr_dev_t, apr_fileperms_t, apr_filetype_e, apr_finfo_t, apr_gid_t,
        apr_ino_t, apr_int32_t, apr_int64_t,
        apr_off_t, apr_pool_t, apr_port_t, apr_time_t, apr_uid_t, apr_uri_t,
        conn_rec, request_rec,
    };
    use crate::apache2::memory::test_utils::with_pool;
    use crate::apache2::connection::ConnectionContext;
    use crate::apache2::connection::test_utils::with_conn_rec;
    use crate::schema::tile::config::ModuleConfig;
    use std::boxed::Box;
    use std::error::Error;
    use std::ops::FnOnce;
    use std::os::raw::{ c_int, c_uint, };
    use std::ptr;

    impl<'r> RequestContext<'r> {
        pub fn create_with_tile_config(
            record: &'r mut request_rec,
            config: &'r ModuleConfig,
        ) -> Result<&'r mut Self, Box<dyn Error>> {
            let connection = unsafe { &mut *(record.connection) };
            let conn_context = ConnectionContext::create_with_tile_config(connection, config)?;
            RequestContext::create(record, conn_context)
        }
    }

    impl apr_uri_t {
        pub fn new() -> apr_uri_t {
            apr_uri_t {
                scheme: ptr::null_mut(),
                hostinfo: ptr::null_mut(),
                user: ptr::null_mut(),
                password: ptr::null_mut(),
                hostname: ptr::null_mut(),
                port_str: ptr::null_mut(),
                path: ptr::null_mut(),
                query: ptr::null_mut(),
                fragment: ptr::null_mut(),
                hostent: ptr::null_mut(),
                port: 0 as apr_port_t,
                _bitfield_align_1: [],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 1usize]),
                __bindgen_padding_0: [0; 5usize],
            }
        }
    }

    impl apr_finfo_t {
        pub fn new() -> apr_finfo_t {
            apr_finfo_t {
                pool: ptr::null_mut(),
                valid: 0 as apr_int32_t,
                protection: 0 as apr_fileperms_t,
                filetype: 0 as apr_filetype_e,
                user: 0 as apr_uid_t,
                group: 0 as apr_gid_t,
                inode: 0 as apr_ino_t,
                device: 0 as apr_dev_t,
                nlink: 0 as apr_int32_t,
                size: 0 as apr_off_t,
                csize: 0 as apr_off_t,
                atime: 0 as apr_time_t,
                mtime: 0 as apr_time_t,
                ctime: 0 as apr_time_t,
                fname: ptr::null(),
                name: ptr::null(),
                filehand: ptr::null_mut(),
            }
        }
    }

    pub fn with_request_rec<F>(func: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce(&mut request_rec) -> Result<(), Box<dyn Error>> {
        let mut record: request_rec = request_rec {
            pool: ptr::null_mut(),
            connection: ptr::null_mut(),
            server: ptr::null_mut(),
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
            main: ptr::null_mut(),
            the_request: ptr::null_mut(),
            assbackwards: 0 as c_int,
            proxyreq: 0 as c_int,
            header_only: 0 as c_int,
            proto_num: 0 as c_int,
            protocol: ptr::null_mut(),
            hostname: ptr::null(),
            request_time: 0 as apr_time_t,
            status_line: ptr::null(),
            status: 0 as c_int,
            method_number: 0 as c_int,
            method: ptr::null(),
            allowed: 0 as apr_int64_t,
            allowed_xmethods: ptr::null_mut(),
            allowed_methods: ptr::null_mut(),
            sent_bodyct: 0 as apr_off_t,
            bytes_sent: 0 as apr_off_t,
            mtime: 0 as apr_time_t,
            range: ptr::null(),
            clength: 0 as apr_off_t,
            chunked: 0 as c_int,
            read_body: 0 as c_int,
            read_chunked: 0 as c_int,
            expecting_100: 0 as c_uint,
            kept_body: ptr::null_mut(),
            body_table: ptr::null_mut(),
            remaining: 0 as apr_off_t,
            read_length: 0 as apr_off_t,
            headers_in: ptr::null_mut(),
            headers_out: ptr::null_mut(),
            err_headers_out: ptr::null_mut(),
            subprocess_env: ptr::null_mut(),
            notes: ptr::null_mut(),
            content_type: ptr::null(),
            handler: ptr::null(),
            content_encoding: ptr::null(),
            content_languages: ptr::null_mut(),
            vlist_validator: ptr::null_mut(),
            user: ptr::null_mut(),
            ap_auth_type: ptr::null_mut(),
            unparsed_uri: ptr::null_mut(),
            uri: ptr::null_mut(),
            filename: ptr::null_mut(),
            canonical_filename: ptr::null_mut(),
            path_info: ptr::null_mut(),
            args: ptr::null_mut(),
            used_path_info: 0 as c_int,
            eos_sent: 0 as c_int,
            per_dir_config: ptr::null_mut(),
            request_config: ptr::null_mut(),
            log: ptr::null(),
            log_id: ptr::null(),
            htaccess: ptr::null(),
            output_filters: ptr::null_mut(),
            input_filters: ptr::null_mut(),
            proto_output_filters: ptr::null_mut(),
            proto_input_filters: ptr::null_mut(),
            no_cache: 0 as c_int,
            no_local_copy: 0 as c_int,
            invoke_mtx: ptr::null_mut(),
            parsed_uri: apr_uri_t::new(),
            finfo: apr_finfo_t::new(),
            useragent_addr: ptr::null_mut(),
            useragent_ip: ptr::null_mut(),
            trailers_in: ptr::null_mut(),
            trailers_out: ptr::null_mut(),
            useragent_host: ptr::null_mut(),
            double_reverse: 0 as c_int,
        };
        with_pool(|pool| {
            record.pool = pool as *mut apr_pool_t;
            with_conn_rec(|connection| {
                record.connection = connection as *mut conn_rec;
                record.server = connection.base_server;
                func(&mut record)
            })
        })
    }
}
