#![allow(unused_unsafe)]

use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS,
    APLOG_ERR, APLOG_NOTICE, DECLINED, HTTP_INTERNAL_SERVER_ERROR, OK,
    apr_pool_t, apr_status_t, conn_rec, request_rec, server_rec,
};
use crate::apache2::connection::ConnectionContext;
use crate::apache2::hook::InvalidArgError;
use crate::apache2::memory::{ alloc, retrieve, };
use crate::apache2::virtual_host::VirtualHostContext;

use std::any::type_name;
use std::boxed::Box;
use std::convert::From;
use std::error::Error;
use std::ffi::{CStr, CString,};
use std::fmt;
use std::io::Write;
use std::os::raw::{c_int, c_void,};
use std::option::Option;
use std::ptr;
use std::result::Result;
use std::string::String;
use std::str::Utf8Error;

pub struct RequestContext<'r> {
    pub record: &'r mut request_rec,
    pub host: &'r mut VirtualHostContext<'r>,
    pub connection: &'r mut ConnectionContext<'r>,
    pub uri: &'r str,
    pub request: Option<Request>,
}

pub enum Request {
    ReportModStats,
    DescribeLayer,
    ServeTileV3(ServeTileRequestV3),
    ServeTileV2(ServeTileRequestV2),
}

pub struct DescribeLayerRequest {
    layer: i32,
}

pub struct ServeTileRequestV3 {
    parameter: String,
    x: i32,
    y: i32,
    z: i32,
    extension: String,
    option: Option<String>,
}

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

    pub fn find_or_create(record: &'r mut request_rec) -> Result<&'r mut Self, Box<dyn Error>> {
        if record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.pool".to_string(),
                reason: "null pointer".to_string(),
            }));
        } else if record.server == ptr::null_mut() {
            return Err(Box::new(InvalidArgError{
                arg: "request_rec.server".to_string(),
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
            log!(APLOG_ERR, record.server, "RequestContext::find_or_create - start");
            let context = match retrieve(&mut *(record.pool), &(Self::get_id(record))) {
                Some(existing_context) => existing_context,
                None => {
                    let server = &mut *(record.server);
                    let pool = &mut *(record.pool);
                    let connection = &mut *(record.connection);
                    let uri = CStr::from_ptr(record.uri).to_str()?;
                    Self::create(record, pool, server, connection, uri)?
                },
            };
            log!(APLOG_ERR, context.record.server, "RequestContext::find_or_create - finish");
            return Ok(context);
        }
    }

    fn create(
        record: &'r mut request_rec,
        record_pool: &'r mut apr_pool_t,
        server: &'r mut server_rec,
        connection: &'r mut conn_rec,
        uri: &'r str,
    ) -> Result<&'r mut Self, Box<dyn Error>> {
        log!(APLOG_ERR, server, "RequestContext::create - start");
        let new_context = alloc::<RequestContext<'r>>(
            record_pool,
            &(Self::get_id(record)),
            Some(drop_request_context),
        )?.0;
        new_context.record = record;
        new_context.host = VirtualHostContext::find_or_create(server)?;
        new_context.connection = ConnectionContext::find_or_create(connection)?;
        new_context.uri = uri;
        log!(APLOG_ERR, new_context.host.record, "RequestContext::create - finish");
        return Ok(new_context);
    }
}

#[no_mangle]
pub unsafe extern fn drop_request_context(context_void: *mut c_void) -> apr_status_t {
    if context_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let context_ptr = context_void as *mut RequestContext;
    log!(APLOG_ERR, (&mut *context_ptr).record.server, "drop_request_context - start");
    let context_ref = &mut *context_ptr;
    drop(context_ref);
    log!(APLOG_ERR, (&mut *context_ptr).record.server, "drop_request_context - finish");
    return APR_SUCCESS as apr_status_t;
}

#[no_mangle]
pub extern "C" fn parse(record_ptr: *mut request_rec) -> c_int {
    if record_ptr == ptr::null_mut() {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    } else {
        unsafe {
            let record = &mut *record_ptr;
            log!(APLOG_ERR, record.server, "slippy::request::parse - start");
            let context = match RequestContext::find_or_create(record) {
                Ok(context) => context,
                Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
            };
            match _parse(context) {
                Ok(request) => {
                    context.request = Some(request);
                    log!(APLOG_ERR, record.server, "slippy::request::parse - finish");
                    return OK as c_int;
                },
                Err(err) => match err {
                    ParseError::Param(err) => {
                        log!(
                            APLOG_NOTICE,
                            record.server,
                            format!("Parameter {} error: {}", err.param, err.reason)
                        );
                        return DECLINED as c_int;
                    },
                    ParseError::Io(why) => {
                        log!(APLOG_ERR, record.server, format!("IO error: {}", why));
                        return HTTP_INTERNAL_SERVER_ERROR as c_int;
                    },
                    ParseError::Utf8(why) => {
                        log!(APLOG_ERR, record.server, format!("UTF8 error: {}", why));
                        return HTTP_INTERNAL_SERVER_ERROR as c_int;
                    },
                },
            }
        }
    }
}

fn _parse(context: &RequestContext) -> Result<Request, ParseError> {
    context.host.trace_file.borrow_mut().write_all(b"slippy::request::_parse - start\n")?;
    write!(context.host.trace_file.borrow_mut(), "slippy::request::_parse - url.path={}\n", context.uri)?;

    // try match stats request
    let module_name = unsafe {
        CStr::from_ptr(crate::TILE_MODULE.name).to_str()?
    };
    let stats_uri = format!("/{}", module_name);
    if context.uri.eq(&stats_uri) {
        context.host.trace_file.borrow_mut().write_all(b"slippy::request::_parse ReportModStats - finish\n")?;
        return Ok(Request::ReportModStats);
    }

    // try match ServeTileV3
    match scan_fmt!(
        context.uri,
        "/{40[^/]}/{d}/{d}/{d}.{255[a-z]}/{10}",
        String, i32, i32, i32, String, String
    ) {
        Ok((parameter, x, y, z, extension, option)) => {
            context.host.trace_file.borrow_mut().write_all(b"slippy::request::_parse ServeTileV3 - finish\n")?;
            return Ok(Request::ServeTileV3(
                ServeTileRequestV3 {
                    parameter,
                    x,
                    y,
                    z,
                    extension,
                    option: if option.is_empty() {
                        None
                    } else {
                        Some(option)
                    },
                }
            ))
        },
        Err(_) => ()
    }

    // try match ServeTileV2
    match scan_fmt!(
        context.uri,
        "/{d}/{d}/{d}.{255[a-z]}/{10}",
        i32, i32, i32, String, String
    ) {
        Ok((x, y, z, extension, option)) => {
            context.host.trace_file.borrow_mut().write_all(b"slippy::request::_parse ServeTileV2 - finish\n")?;
            return Ok(Request::ServeTileV2(
                ServeTileRequestV2 {
                    x,
                    y,
                    z,
                    extension,
                    option: if option.is_empty() {
                        None
                    } else {
                        Some(option)
                    },
                }
            ))
        },
        Err(_) => ()
    }
    context.host.trace_file.borrow_mut().write_all(b"slippy::request::_parse no matches - finish\n")?;
    return Err(ParseError::Param(
        InvalidParameterError {
            param: "uri".to_string(),
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
    reason: String,
}

impl Error for InvalidParameterError {}

impl fmt::Display for InvalidParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parameter {} is invalid: {}", self.param, self.reason)
    }
}
