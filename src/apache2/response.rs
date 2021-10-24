use crate::apache2::bindings::{
    ap_rwrite, ap_set_content_type, ap_set_content_length,
    apr_palloc, apr_psprintf, apr_table_setn, apr_table_mergen, apr_rfc822_date, request_rec
};
use crate::apache2::request::RequestContext;

use http::header::{ HeaderName, HeaderValue };
use http::status::StatusCode;
use mime::Mime;

use std::ffi::{ CString, NulError };
use std::option::Option;
use std::os::raw::c_char;


#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub bytes_written: u32,
    pub http_header: Option<(HeaderName, HeaderValue)>,
}

pub struct ResponseContext<'r> {
    pub request_record: &'r mut request_rec,
    //pub request_context: &'r RequestContext<'r>,
}

impl<'r> ResponseContext<'r> {
    pub fn from(request: &'r mut RequestContext<'r>) -> ResponseContext<'r> {
        ResponseContext {
            request_record: request.record,
            //request_context: request,
        }
    }

    pub fn append_http_header(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), NulError> {
        let c_key = CString::new(key)?;
        let c_value = CString::new(value)?;
        unsafe {
            apr_table_mergen(
                self.request_record.headers_out,
                c_key.as_c_str().as_ptr(),
                apr_psprintf(
                    self.request_record.pool,
                    "%s".as_ptr() as *const c_char,
                    c_value.as_c_str().as_ptr()
                )
            );
        }
        Ok(())
    }

    pub fn set_http_header(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), NulError> {
        let c_key = CString::new(key)?;
        let c_value = CString::new(value)?;
        unsafe {
            apr_table_setn(
                self.request_record.headers_out,
                c_key.as_c_str().as_ptr(),
                apr_psprintf(
                    self.request_record.pool,
                    "%s".as_ptr() as *const c_char,
                    c_value.as_c_str().as_ptr()
                )
            );
        }
        Ok(())
    }

    pub fn set_content_type(
        &mut self,
        mime: &Mime,
    ) -> Result<(), NulError> {
        let mime_str = CString::new(mime.essence_str())?;
        unsafe {
            ap_set_content_type(
                self.request_record,
                mime_str.as_c_str().as_ptr(),
            );
        }
        Ok(())
    }

    pub fn set_content_length(
        &mut self,
        length: i64,
    ) -> Result<(), NulError> {
        unsafe {
            ap_set_content_length(
                self.request_record,
                length,
            );
        }
        Ok(())
    }

    pub fn write<T: AsRef<[u8]>>(payload: T) -> i32 {
        return 0;
    }
}
