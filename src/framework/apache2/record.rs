use crate::binding::apache2::{ apr_pool_t, conn_rec, process_rec, request_rec, server_rec, };
use crate::schema::apache2::error::InvalidRecordError;

use std::ffi::CStr;
use std::option::Option;
use std::ptr;


pub trait RequestRecord {
    fn get_connection_record<'s>(self: &'s Self) -> Result<&'s conn_rec, InvalidRecordError>;

    fn get_server_record<'s>(self: &'s Self) -> Result<&'s server_rec, InvalidRecordError>;

    fn get_pool<'p>(&'p self) -> Result<&'p mut apr_pool_t, InvalidRecordError>;
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

    fn get_pool<'p>(&'p self) -> Result<&'p mut apr_pool_t, InvalidRecordError> {
        if self.pool == ptr::null_mut() {
            Err(InvalidRecordError::new(
                self.pool,
                "request_rec.pool field is a null pointer",
            ))
        } else {
            Ok(unsafe { self.pool.as_mut().unwrap() })
        }
    }
}

pub trait ConnectionRecord {
    fn get_server_record<'s>(self: &'s Self) -> Result<&'s server_rec, InvalidRecordError>;

    fn get_pool<'p>(&'p self) -> Result<&'p mut apr_pool_t, InvalidRecordError>;
}

impl ConnectionRecord for conn_rec {
    fn get_server_record<'s>(self: &'s Self) -> Result<&'s server_rec, InvalidRecordError> {
        if self.base_server == ptr::null_mut() {
            Err(InvalidRecordError::new(
                self as *const conn_rec,
                "base_server field is null pointer",
            ))
        } else {
            Ok(unsafe { &(*self.base_server) } )
        }
    }

    fn get_pool<'p>(&'p self) -> Result<&'p mut apr_pool_t, InvalidRecordError> {
        if self.pool == ptr::null_mut() {
            Err(InvalidRecordError::new(
                self.pool,
                "conn_rec.pool field is a null pointer",
            ))
        } else {
            Ok(unsafe { self.pool.as_mut().unwrap() })
        }
    }
}

pub trait ServerRecord {
    fn get_host_name<'s>(&'s self) -> Option<&'s str>;

    fn get_pool<'s>(&'s self) -> Result<&'s mut apr_pool_t, InvalidRecordError>;

    fn get_process_record<'s>(&'s self) -> Result<&'s process_rec, InvalidRecordError>;
}

impl ServerRecord for server_rec {
    fn get_host_name<'s>(&'s self) -> Option<&'s str> {
        if self.server_hostname == ptr::null_mut() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.server_hostname).to_str().unwrap() })
        }
    }

    fn get_pool<'s>(&'s self) -> Result<&'s mut apr_pool_t, InvalidRecordError> {
        let proc_record = self.get_process_record().unwrap();
        if proc_record.pool == ptr::null_mut() {
            Err(InvalidRecordError::new(
                proc_record as *const process_rec,
                "process_rec.pool field is null pointer",
            ))
        } else {
            proc_record.get_pool()
        }
    }

    fn get_process_record<'s>(&'s self) -> Result<&'s process_rec, InvalidRecordError> {
        if self.process == ptr::null_mut() {
            return Err(InvalidRecordError::new(
                self.process,
                "server_rec.process field is a null pointer",
            ));
        }
        let proc_record = unsafe { self.process.as_mut().unwrap() };
        if proc_record.pool == ptr::null_mut() {
            return Err(InvalidRecordError::new(
                proc_record as *const process_rec,
                "server_rec.process.pool field is a null pointer",
            ));
        }
        Ok(proc_record)
    }
}

pub trait ProcessRecord {
    fn get_pool<'p>(&'p self) -> Result<&'p mut apr_pool_t, InvalidRecordError>;
}

impl ProcessRecord for process_rec {
    fn get_pool<'p>(&'p self) -> Result<&'p mut apr_pool_t, InvalidRecordError> {
        if self.pool == ptr::null_mut() {
            Err(InvalidRecordError::new(
                self.pool,
                "process_rec.pool field is a null pointer",
            ))
        } else {
            Ok(unsafe { self.pool.as_mut().unwrap() })
        }
    }
}


#[cfg(test)]
pub mod test_utils {
    use crate::binding::apache2::{
        __BindgenBitfieldUnit, ap_conn_keepalive_e, ap_logconf,
        apr_dev_t, apr_fileperms_t, apr_filetype_e, apr_finfo_t, apr_gid_t,
        apr_ino_t, apr_int32_t, apr_int64_t, apr_interval_time_t,
        apr_off_t, apr_pool_t, apr_port_t, apr_time_t, apr_uid_t, apr_uri_t,
        conn_rec, process_rec, request_rec, server_rec,
    };
    use crate::framework::apache2::memory::test_utils::with_pool;
    use std::boxed::Box;
    use std::error::Error;
    use std::ops::FnOnce;
    use std::os::raw::{ c_char, c_int, c_long, c_uint, };
    use std::ptr;

    impl ap_logconf {
        pub fn new() -> ap_logconf {
            ap_logconf {
                module_levels: ptr::null_mut(),
                level: 0,
            }
        }
    }

    impl server_rec {
        pub fn new() -> server_rec {
            server_rec {
                process: ptr::null_mut(),
                next: ptr::null_mut(),
                error_fname: ptr::null_mut(),
                error_log: ptr::null_mut(),
                log: ap_logconf::new(),
                module_config: ptr::null_mut(),
                lookup_defaults: ptr::null_mut(),
                defn_name: ptr::null(),
                defn_line_number: 0,
                is_virtual: '\0' as c_char,
                port: 0 as apr_port_t,
                server_scheme: ptr::null(),
                server_admin: ptr::null_mut(),
                server_hostname: cstr!("localhost") as *mut i8,
                addrs: ptr::null_mut(),
                timeout: 0 as apr_interval_time_t,
                keep_alive_timeout: 0 as apr_interval_time_t,
                keep_alive_max: 0 as c_int,
                keep_alive: 0 as c_int,
                names: ptr::null_mut(),
                wild_names: ptr::null_mut(),
                path: ptr::null(),
                pathlen: 0 as c_int,
                limit_req_line: 0 as c_int,
                limit_req_fieldsize: 0 as c_int,
                limit_req_fields: 0 as c_int,
                context: ptr::null_mut(),
                _bitfield_align_1: [],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 1usize]),
                __bindgen_padding_0: [0; 7usize],
            }
        }
    }

    pub fn with_server_rec<F>(func: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce(&mut server_rec) -> Result<(), Box<dyn Error>> {
        let mut process: process_rec = process_rec {
            pool: ptr::null_mut(),
            pconf: ptr::null_mut(),
            short_name: cstr!("test"),
            argv: ptr::null_mut(),
            argc: 0,
        };
        let mut record: server_rec = server_rec::new();
        record.process = &mut process;
        with_pool(|proc_pool| {
            process.pool = proc_pool as *mut apr_pool_t;
            with_pool(|conf_pool| {
                process.pconf = conf_pool as *mut apr_pool_t;
                func(&mut record)
            })
        })
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
