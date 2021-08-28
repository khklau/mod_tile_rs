#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS,
    DECLINED, HTTP_INTERNAL_SERVER_ERROR, OK,
    apr_pool_t, apr_status_t, conn_rec, request_rec,
};
use crate::apache2::connection::ConnectionContext;
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::{ alloc, retrieve, };
use crate::apache2::virtual_host::VirtualHostContext;

use scan_fmt::scan_fmt;

use std::any::type_name;
use std::boxed::Box;
use std::convert::From;
use std::error::Error;
use std::ffi::{CStr, CString,};
use std::fmt;
use std::os::raw::{c_int, c_void,};
use std::option::Option;
use std::ptr;
use std::result::Result;
use std::string::String;
use std::str::Utf8Error;

pub struct RequestContext<'r> {
    pub record: &'r mut request_rec,
    pub connection: &'r mut ConnectionContext<'r>,
    pub uri: &'r str,
    pub request: Option<Request>,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Request {
    ReportModStats,
    DescribeLayer,
    ServeTileV3(ServeTileRequestV3),
    ServeTileV2(ServeTileRequestV2),
}

pub struct DescribeLayerRequest {
    layer: i32,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct ServeTileRequestV3 {
    parameter: String,
    x: i32,
    y: i32,
    z: i32,
    extension: String,
    option: Option<String>,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct ServeTileRequestV2 {
    x: i32,
    y: i32,
    z: i32,
    extension: String,
    option: Option<String>,
}

impl<'r> RequestContext<'r> {

    pub fn get_id(record: &request_rec) -> CString {
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

    pub fn find_or_create(record: &'r mut request_rec) -> Result<&'r mut Self, Box<dyn Error>> {
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        } else if record.connection == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.connection".to_string(),
                reason: "null pointer".to_string(),
            }));
        } else if record.uri == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.uri".to_string(),
                reason: "null pointer".to_string(),
            }));
        }
        unsafe {
            info!(record.server, "RequestContext::find_or_create - start");
            let context = match retrieve(&mut *(record.pool), &(Self::get_id(record))) {
                Some(existing_context) => existing_context,
                None => {
                    let pool = &mut *(record.pool);
                    let connection = &mut *(record.connection);
                    let uri = CStr::from_ptr(record.uri).to_str()?;
                    Self::create(record, pool, connection, uri)?
                },
            };
            info!(context.record.server, "RequestContext::find_or_create - finish");
            return Ok(context);
        }
    }

    fn create(
        record: &'r mut request_rec,
        record_pool: &'r mut apr_pool_t,
        connection: &'r mut conn_rec,
        uri: &'r str,
    ) -> Result<&'r mut Self, Box<dyn Error>> {
        let conn_context = ConnectionContext::find_or_create(connection)?;
        info!(conn_context.host.record, "RequestContext::create - start");
        let new_context = alloc::<RequestContext<'r>>(
            record_pool,
            &(Self::get_id(record)),
            Some(drop_request_context),
        )?.0;
        new_context.record = record;
        new_context.connection = conn_context;
        new_context.uri = uri;
        info!(new_context.connection.host.record, "RequestContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
pub unsafe extern fn drop_request_context(context_void: *mut c_void) -> apr_status_t {
    if context_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let context_ptr = context_void as *mut RequestContext;
    info!((&mut *context_ptr).record.server, "drop_request_context - start");
    let context_ref = &mut *context_ptr;
    drop(context_ref);
    info!((&mut *context_ptr).record.server, "drop_request_context - finish");
    return APR_SUCCESS as apr_status_t;
}

#[no_mangle]
pub extern "C" fn parse(record_ptr: *mut request_rec) -> c_int {
    if record_ptr == ptr::null_mut() {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    } else {
        unsafe {
            let record = &mut *record_ptr;
            info!(record.server, "slippy::request::parse - start");
            let context = match RequestContext::find_or_create(record) {
                Ok(context) => context,
                Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
            };
            match _parse(context) {
                Ok(request) => {
                    context.request = Some(request);
                    info!(record.server, "slippy::request::parse - finish");
                    return OK as c_int;
                },
                Err(err) => match err {
                    ParseError::Param(err) => {
                        info!(record.server, "Parameter {} error: {}", err.param, err.reason);
                        return DECLINED as c_int;
                    },
                    ParseError::Io(why) => {
                        info!(record.server, "IO error: {}", why);
                        return HTTP_INTERNAL_SERVER_ERROR as c_int;
                    },
                    ParseError::Utf8(why) => {
                        info!(record.server, "UTF8 error: {}", why);
                        return HTTP_INTERNAL_SERVER_ERROR as c_int;
                    },
                },
            }
        }
    }
}

fn _parse(context: &RequestContext) -> Result<Request, ParseError> {
    info!(context.get_host().record, "slippy::request::_parse - uri={}", context.uri);

    let base_url = context.get_host().tile_config.base_url.as_str();
    let request_url = if let Some(found) = context.uri.find(base_url) {
        let after_base = found + base_url.len();
        if let Some(input_url) = context.uri.get(after_base..) {
            input_url
        } else {
            ""
        }
    } else {
        context.uri
    };

    // try match stats request
    let module_name = unsafe {
        CStr::from_ptr(crate::TILE_MODULE.name).to_str()?
    };
    let stats_uri = format!("/{}", module_name);
    if request_url.eq(&stats_uri) {
        info!(context.get_host().record, "slippy::request::_parse - parsed ReportModStats");
        return Ok(Request::ReportModStats);
    }

    // try match ServeTileV3 with option
    match scan_fmt!(
        request_url,
        "/{40[^/]}/{d}/{d}/{d}.{255[a-z]}/{10[^/]}",
        String, i32, i32, i32, String, String
    ) {
        Ok((parameter, x, y, z, extension, option)) => {
            info!(context.get_host().record, "slippy::request::_parse - parsed ServeTileV3 with option");
            return Ok(Request::ServeTileV3(
                ServeTileRequestV3 {
                    parameter,
                    x,
                    y,
                    z,
                    extension,
                    option: Some(option)
                }
            ));
        },
        Err(_) => ()
    }

    // try match ServeTileV3 no option
    match scan_fmt!(
        request_url,
        "/{40[^/]}/{d}/{d}/{d}.{255[a-z]}{///?/}",
        String, i32, i32, i32, String
    ) {
        Ok((parameter, x, y, z, extension)) => {
            info!(context.get_host().record, "slippy::request::_parse - parsed ServeTileV3 no option");
            return Ok(Request::ServeTileV3(
                ServeTileRequestV3 {
                    parameter,
                    x,
                    y,
                    z,
                    extension,
                    option: None,
                }
            ));
        },
        Err(_) => ()
    }

    // try match ServeTileV2 with option
    match scan_fmt!(
        request_url,
        "/{d}/{d}/{d}.{255[a-z]}/{10[^/]}",
        i32, i32, i32, String, String
    ) {
        Ok((x, y, z, extension, option)) => {
            info!(context.get_host().record, "slippy::request::_parse - parsed ServeTileV2");
            return Ok(Request::ServeTileV2(
                ServeTileRequestV2 {
                    x,
                    y,
                    z,
                    extension,
                    option: Some(option),
                }
            ));
        },
        Err(_) => ()
    }

    // try match ServeTileV2 no option
    match scan_fmt!(
        request_url,
        "/{d}/{d}/{d}.{255[a-z]}{///?/}",
        i32, i32, i32, String
    ) {
        Ok((x, y, z, extension)) => {
            info!(context.get_host().record, "slippy::request::_parse - parsed ServeTileV2");
            return Ok(Request::ServeTileV2(
                ServeTileRequestV2 {
                    x,
                    y,
                    z,
                    extension,
                    option: None,
                }
            ));
        },
        Err(_) => ()
    }

    info!(context.get_host().record, "slippy::request::_parse - URI {} does not match any known request types", request_url);
    return Err(ParseError::Param(
        InvalidParameterError {
            param: "uri".to_string(),
            value: request_url.to_string(),
            reason: "Does not match any known request types".to_string(),
        }
    ));
}

#[derive(Debug)]
enum ParseError {
    Param(InvalidParameterError),
    Io(std::io::Error),
    Utf8(Utf8Error),
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::Param(err) => return Some(err),
            ParseError::Io(err) => return Some(err),
            ParseError::Utf8(err) => return Some(err),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Param(err) => return write!(f, "{}", err),
            ParseError::Io(err) => return write!(f, "{}", err),
            ParseError::Utf8(err) => return write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        return ParseError::Io(error);
    }
}

impl From<Utf8Error> for ParseError {
    fn from(error: Utf8Error) -> Self {
        return ParseError::Utf8(error);
    }
}

#[derive(Debug)]
struct InvalidParameterError {
    param: String,
    value: String,
    reason: String,
}

impl Error for InvalidParameterError {}

impl fmt::Display for InvalidParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parameter {} value {} is invalid: {}", self.param, self.value, self.reason)
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::apache2::bindings::{
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

#[cfg(test)]
mod tests {
    use super::test_utils::with_request_rec;
    use super::*;
    use crate::tile::config::TileConfig;
    use std::boxed::Box;
    use std::error::Error;

    #[test]
    fn test_parse_report_mod_stats() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new(format!("{}/mod_tile_rs", tile_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::find_or_create(record)?;
            let request = _parse(context)?;
            assert!(matches!(request, Request::ReportModStats));
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_with_option() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new(format!("{}/foo/7/8/9.png/bar", tile_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::find_or_create(record)?;
            let actual_request = _parse(context)?;
            let expected_request = Request::ServeTileV3(
                ServeTileRequestV3 {
                    parameter: String::from("foo"),
                    x: 7,
                    y: 8,
                    z: 9,
                    extension: String::from("png"),
                    option: Some(String::from("bar")),
                }
            );
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_no_option_with_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new(format!("{}/foo/7/8/9.png/", tile_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::find_or_create(record)?;
            let actual_request = _parse(context)?;
            let expected_request = Request::ServeTileV3(
                ServeTileRequestV3 {
                    parameter: String::from("foo"),
                    x: 7,
                    y: 8,
                    z: 9,
                    extension: String::from("png"),
                    option: None,
                }
            );
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_no_option_no_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new(format!("{}/foo/7/8/9.png", tile_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::find_or_create(record)?;
            let actual_request = _parse(context)?;
            let expected_request = Request::ServeTileV3(
                ServeTileRequestV3 {
                    parameter: String::from("foo"),
                    x: 7,
                    y: 8,
                    z: 9,
                    extension: String::from("png"),
                    option: None,
                }
            );
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_with_option() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new(format!("{}/1/2/3.jpg/blah", tile_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::find_or_create(record)?;
            let actual_request = _parse(context)?;
            let expected_request = Request::ServeTileV2(
                ServeTileRequestV2 {
                    x: 1,
                    y: 2,
                    z: 3,
                    extension: String::from("jpg"),
                    option: Some(String::from("blah")),
                }
            );
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_no_option_with_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new(format!("{}/1/2/3.jpg/", tile_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::find_or_create(record)?;
            let actual_request = _parse(context)?;
            let expected_request = Request::ServeTileV2(
                ServeTileRequestV2 {
                    x: 1,
                    y: 2,
                    z: 3,
                    extension: String::from("jpg"),
                    option: None,
                }
            );
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_no_option_no_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new(format!("{}/1/2/3.jpg", tile_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::find_or_create(record)?;
            let actual_request = _parse(context)?;
            let expected_request = Request::ServeTileV2(
                ServeTileRequestV2 {
                    x: 1,
                    y: 2,
                    z: 3,
                    extension: String::from("jpg"),
                    option: None,
                }
            );
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }
}
