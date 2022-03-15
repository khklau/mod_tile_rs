use crate::apache2::memory::{ access_pool_object, alloc, retrieve, };

use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS,
    apr_status_t, conn_rec, request_rec, server_rec,
};
use crate::schema::apache2::error::InvalidRecordError;

use snowflake::SnowflakeIdGenerator;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::{CStr, CString,};
use std::os::raw::c_void;
use std::ptr;
use std::result::Result;


pub trait RequestRecord {
    fn get_connection_record<'s>(self: &'s Self) -> Result<&'s conn_rec, InvalidRecordError>;

    fn get_server_record<'s>(self: &'s Self) -> Result<&'s server_rec, InvalidRecordError>;
}

impl RequestRecord for request_rec {
    fn get_connection_record<'s>(self: &'s Self) -> Result<&'s conn_rec, InvalidRecordError> {
        if self.connection == ptr::null_mut() {
            Err(InvalidRecordError::new(
                self as *const request_rec,
                "request_rec.connection field is null pointer",
            ))
        } else {
            Ok(unsafe { &(*self.connection) } )
        }
    }

    fn get_server_record<'s>(self: &'s Self) -> Result<&'s server_rec, InvalidRecordError> {
        if self.server == ptr::null_mut() {
            Err(InvalidRecordError::new(
                self as *const request_rec,
                "request_rec.server field is null pointer",
            ))
        } else {
            Ok(unsafe { &(*self.server) } )
        }
    }
}

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
    use crate::binding::apache2::{
        __BindgenBitfieldUnit, apr_dev_t, apr_fileperms_t, apr_filetype_e, apr_finfo_t, apr_gid_t,
        apr_ino_t, apr_int32_t, apr_int64_t,
        apr_off_t, apr_pool_t, apr_port_t, apr_time_t, apr_uid_t, apr_uri_t,
        conn_rec, request_rec,
    };
    use crate::apache2::memory::test_utils::with_pool;
    use crate::apache2::connection::test_utils::with_conn_rec;
    use std::boxed::Box;
    use std::error::Error;
    use std::ops::FnOnce;
    use std::os::raw::{ c_int, c_uint, };
    use std::ptr;

    impl<'r> Apache2Request<'r> {
        pub fn create_with_tile_config(
            record: &'r request_rec,
        ) -> Result<&'r mut Self, Box<dyn Error>> {
            Apache2Request::create(record)
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
