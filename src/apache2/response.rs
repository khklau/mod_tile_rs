use crate::apache2::bindings::request_rec;
#[cfg(not(test))]
use crate::apache2::bindings::{
    ap_rwrite, ap_rflush, ap_set_content_type, ap_set_content_length,
    apr_psprintf, apr_table_setn, apr_table_mergen,
};
use crate::apache2::request::RequestContext;
use crate::apache2::virtual_host::VirtualHostContext;
use crate::schema::apache2::error::ResponseWriteError;

use http::header::{ HeaderMap, HeaderName, HeaderValue, ToStrError, };
use http::status::StatusCode;
use mime::Mime;

#[cfg(not(test))]
use std::ffi::{ CString, c_void };
use std::mem::size_of;
use std::option::Option;


#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub bytes_written: usize,
    pub http_headers: HeaderMap,
}

pub struct ResponseContext<'r> {
    pub request: &'r mut RequestContext<'r>,
    apache2_writer: Apache2Writer,
    writer: Option<&'r mut dyn Writer<ElementType = u8>>,
}

impl<'r> ResponseContext<'r> {
    pub fn from(request: &'r mut RequestContext<'r>) -> ResponseContext<'r> {
        ResponseContext {
            request: request,
            apache2_writer: Apache2Writer { },
            writer: None,
        }
    }

    pub fn get_host(&self) -> &VirtualHostContext {
        self.request.get_host()
    }

    #[cfg(not(test))]
    pub fn append_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        let c_key = CString::new(key.as_str()).unwrap();
        let c_value = CString::new(value.to_str()?).unwrap();
        debug!(
            self.request.get_host().record,
            "ResponseContext::append_http_header - appending {} - {}",
            c_key.to_str().unwrap(),
            c_value.to_str().unwrap()
        );
        unsafe {
            apr_table_mergen(
                self.request.record.headers_out,
                apr_psprintf(
                    self.request.record.pool,
                    cstr!("%s"),
                    c_key.as_c_str().as_ptr(),
                ),
                apr_psprintf(
                    self.request.record.pool,
                    cstr!("%s"),
                    c_value.as_c_str().as_ptr()
                )
            );
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn append_http_header(
        &mut self,
        _key: &HeaderName,
        _value: &HeaderValue,
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
        debug!(
            self.request.get_host().record,
            "ResponseContext::set_http_header - setting {} - {}",
            c_key.to_str().unwrap(),
            c_value.to_str().unwrap()
        );
        unsafe {
            apr_table_setn(
                self.request.record.headers_out,
                apr_psprintf(
                    self.request.record.pool,
                    cstr!("%s"),
                    c_key.as_c_str().as_ptr(),
                ),
                apr_psprintf(
                    self.request.record.pool,
                    cstr!("%s"),
                    c_value.as_c_str().as_ptr(),
                )
            );
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn set_http_header(
        &mut self,
        _key: &HeaderName,
        _value: &HeaderValue,
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
                self.request.record,
                mime_str.as_c_str().as_ptr(),
            );
        }
    }

    #[cfg(test)]
    pub fn set_content_type(
        &mut self,
        _mime: &Mime,
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
                self.request.record,
                length as i64,
            );
        }
    }

    #[cfg(test)]
    pub fn set_content_length(
        &mut self,
        _length: usize,
    ) -> () {
        ()
    }

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
            debug!(self.request.get_host().record, "ResponseContext::write_content - writing slice {}", String::from_utf8_lossy(payload_slice));
            let result = writer.write(
                payload_slice.as_ptr(),
                payload_slice.len(),
                self.request.record
            );
            debug!(self.request.get_host().record, "ResponseContext::write_content - write result {}", result);
            if result >= 0 {
                let elements_written = (result as usize) / size_of::<u8>();
                payload_slice = payload_slice.split_at(elements_written).1;
            } else {
                return Err(ResponseWriteError { error_code: result });
            }
        }
        Ok(payload.as_ref().len())
    }

    #[cfg(not(test))]
    pub fn flush_response(&mut self) -> Result<(), ResponseWriteError> {
        let result = unsafe { ap_rflush(self.request.record) };
        if result < 0 {
            return Err(ResponseWriteError { error_code: result });
        } else {
            return Ok(())
        }
    }

    #[cfg(test)]
    pub fn flush_response(&mut self) -> Result<(), ResponseWriteError> {
        Ok(())
    }
}

trait Writer {
    type ElementType;

    fn write(
        &mut self,
        buffer: *const Self::ElementType,
        length: usize,
        record: &mut request_rec,
    ) -> i32;
}

struct Apache2Writer { }

impl Writer for Apache2Writer {
    type ElementType = u8;

    #[cfg(not(test))]
    fn write(
        &mut self,
        buffer: *const u8,
        length: usize,
        record: &mut request_rec,
    ) -> i32 {
        unsafe {
            ap_rwrite(
                buffer as *const c_void,
                length as i32,
                record
            )
        }
    }

    #[cfg(test)]
    fn write(
        &mut self,
        _buffer: *const u8,
        length: usize,
        _record: &mut request_rec,
    ) -> i32 {
        length as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apache2::request::test_utils::with_request_rec;
    use crate::schema::tile::config::TileConfig;

    use std::cmp::min;
    use std::collections::VecDeque;
    use std::error::Error;
    use std::ffi::CString;
    use std::slice;
    struct TestWriter {
        default_length: usize,
        allowed_lengths: VecDeque<usize>,
        written_payload: Vec<u8>,
        written_lengths: Vec<usize>,
    }

    impl<'w> Writer for TestWriter {
        type ElementType = u8;

        fn write(
            &mut self,
            payload: *const u8,
            length: usize,
            _record: &mut request_rec,
        ) -> i32 {
            let allowed_length = if self.allowed_lengths.is_empty() {
                self.default_length
            } else {
                let length = self.allowed_lengths.front().unwrap().clone();
                self.allowed_lengths.pop_front();
                length
            };
            let write_length = min(length, allowed_length);
            let payload_slice = unsafe { slice::from_raw_parts(payload, write_length) };
            self.written_payload.extend_from_slice(payload_slice);
            self.written_lengths.push(write_length);
            write_length as i32
        }
    }

    #[test]
    fn test_write_content_small_lengths() -> Result<(), Box<dyn Error>> {
        let mut writer = TestWriter {
            default_length: 1,
            allowed_lengths: vec![2, 2, 2, 2].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &config)?;
            let mut context = ResponseContext::from(request_context);
            context.writer = Some(&mut writer);
            context.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![2, 2, 2, 1], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }

    #[test]
    fn test_write_content_large_lengths() -> Result<(), Box<dyn Error>> {
        let mut writer = TestWriter {
            default_length: 1,
            allowed_lengths: vec![128].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &config)?;
            let mut context = ResponseContext::from(request_context);
            context.writer = Some(&mut writer);
            context.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![expected_payload.len()], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }

    #[test]
    fn test_write_content_paused_writes() -> Result<(), Box<dyn Error>> {
        let mut writer = TestWriter {
            default_length: 1,
            allowed_lengths: vec![2, 0, 4, 0, 2].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &config)?;
            let mut context = ResponseContext::from(request_context);
            context.writer = Some(&mut writer);
            context.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![2, 0, 4, 0, 1], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }

    #[test]
    fn test_write_content_delayed_writes() -> Result<(), Box<dyn Error>> {
        let mut writer = TestWriter {
            default_length: 1,
            allowed_lengths: vec![0, 0, 4, 4].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &config)?;
            let mut context = ResponseContext::from(request_context);
            context.writer = Some(&mut writer);
            context.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![0, 0, 4, 3], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }
}