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
use std::mem::size_of;
use std::option::Option;
use std::os::raw::{ c_char, c_int, c_void as raw_void };


#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub bytes_written: usize,
    pub http_headers: HeaderMap,
}

pub struct ResponseContext<'r> {
    pub request_record: &'r mut request_rec,
    apache2_writer: Apache2Writer,
    writer: Option<&'r mut dyn Writer<ElementType = u8>>,
}

impl<'r> ResponseContext<'r> {
    pub fn from(request: &'r mut RequestContext<'r>) -> ResponseContext<'r> {
        ResponseContext {
            request_record: request.record,
            apache2_writer: Apache2Writer { },
            writer: None,
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
        let writer: &mut dyn Writer<ElementType = u8> = match &mut self.writer {
            Some(obj) => &mut *(*obj),
            None => &mut self.apache2_writer,
        };
        let mut payload_slice = payload.as_ref();
        while payload_slice.len() > 0 {
            let result = writer.write(
                payload_slice,
                self.request_record
            );
            if result >= 0 {
                let elements_written = (result as usize) / size_of::<u8>();
                payload_slice = payload_slice.split_at(elements_written).1;
            } else {
                return Err(ResponseWriteError { error_code: result });
            }
        }
        Ok(payload.as_ref().len())
    }

    #[cfg(test)]
    pub fn write_content<T: AsRef<[u8]>>(
        &mut self,
        payload: T,
    ) -> Result<usize, ResponseWriteError> {
        Ok(0)
    }
}

trait Writer {
    type ElementType;

    fn write(
        &mut self,
        buffer: &[Self::ElementType],
        record: &mut request_rec,
    ) -> i32;
}

struct Apache2Writer { }

impl Writer for Apache2Writer {
    type ElementType = u8;

    #[cfg(not(test))]
    fn write(
        &mut self,
        buffer: &[u8],
        record: &mut request_rec,
    ) -> i32 {
        unsafe {
            ap_rwrite(
                buffer.as_ptr() as *const c_void,
                buffer.len() as i32,
                record
            )
        }
    }

    #[cfg(test)]
    fn write(
        &mut self,
        buffer: &[u8],
        record: &mut request_rec,
    ) -> i32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestWriter {
        write_len: usize,
        written_payload: Vec<u8>,
    }

    impl<'w> Writer for TestWriter {
        type ElementType = u8;

        fn write(
            &mut self,
            payload_slice: &[u8],
            _record: &mut request_rec,
        ) -> i32 {
            self.written_payload.extend_from_slice(payload_slice);
            payload_slice.len() as i32
        }
    }
}