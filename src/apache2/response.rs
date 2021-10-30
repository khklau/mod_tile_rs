use crate::apache2::bindings::{
    ap_rwrite, ap_set_content_type, ap_set_content_length,
    apr_psprintf, apr_table_setn, apr_table_mergen, request_rec
};
use crate::apache2::error::ResponseWriteError;
use crate::apache2::request::RequestContext;

use http::header::{ HeaderMap, HeaderName, HeaderValue, ToStrError, };
use http::status::StatusCode;
use mime::Mime;

use std::ffi::{ CString, c_void };
use std::os::raw::c_char;


#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub bytes_written: usize,
    pub http_headers: HeaderMap,
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

    #[cfg(not(test))]
    pub fn append_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        let c_key = CString::new(key.as_str()).unwrap();
        let c_value = CString::new(value.to_str()?).unwrap();
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

    #[cfg(test)]
    pub fn append_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        Ok(())
    }

    #[cfg(not(test))]
    pub fn set_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        let c_key = CString::new(key.as_str()).unwrap();
        let c_value = CString::new(value.to_str()?).unwrap();
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

    #[cfg(test)]
    pub fn set_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        Ok(())
    }

    #[cfg(not(test))]
    pub fn set_content_type(
        &mut self,
        mime: &Mime,
    ) -> () {
        let mime_str = CString::new(mime.essence_str()).unwrap();
        unsafe {
            ap_set_content_type(
                self.request_record,
                mime_str.as_c_str().as_ptr(),
            );
        }
    }

    #[cfg(test)]
    pub fn set_content_type(
        &mut self,
        mime: &Mime,
    ) -> () {
        ()
    }

    #[cfg(not(test))]
    pub fn set_content_length(
        &mut self,
        length: usize,
    ) -> () {
        unsafe {
            ap_set_content_length(
                self.request_record,
                length as i64,
            );
        }
    }

    #[cfg(test)]
    pub fn set_content_length(
        &mut self,
        length: usize,
    ) -> () {
        ()
    }

    #[cfg(not(test))]
    pub fn write_content<T: AsRef<[u8]>>(
        &mut self,
        payload: T,
    ) -> Result<usize, ResponseWriteError> {
        let mut payload_slice = payload.as_ref();
        let mut written_length = 0;
        let target_length = payload.as_ref().len();

        // WARNING: The code below is almost C style pointer arithmetic, it needs thorough reviews
        while written_length < target_length {
            let result = unsafe {
                ap_rwrite(
                    payload_slice.as_ptr() as *const c_void,
                    (target_length - written_length) as i32,
                    self.request_record
                )
            };
            if result >= 0 {
                written_length += result as usize;
                payload_slice = payload_slice.split_at(result as usize).1;
            } else {
                return Err(ResponseWriteError { error_code: result });
            }
        }
        Ok(written_length)
    }

    #[cfg(test)]
    pub fn write_content<T: AsRef<[u8]>>(
        &mut self,
        payload: T,
    ) -> Result<usize, ResponseWriteError> {
        Ok(0)
    }
}
