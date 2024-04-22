use crate::binding::apache2::request_rec;
#[cfg(not(test))]
use crate::binding::apache2::{
    ap_rwrite, ap_rflush, ap_set_content_type, ap_set_content_length,
    apr_psprintf, apr_table_setn, apr_table_mergen,
};
use crate::schema::communication::error::ResponseWriteError;
use crate::schema::http::encoding::ContentEncoding;
use crate::io::communication::interface::HttpResponseWriter;
#[cfg(not(test))]
use crate::framework::apache2::record::RequestRecord;

use http::header::{ HeaderName, HeaderValue, ToStrError, };
use mime::Mime;

#[cfg(not(test))]
use std::ffi::{ CString, c_void };


#[cfg(not(test))]
impl HttpResponseWriter for request_rec {

    fn append_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        let c_key = CString::new(key.as_str()).unwrap();
        let c_value = CString::new(value.to_str()?).unwrap();
        debug!(
            self.get_server_record().unwrap(),
            "request_rec::append_http_header - appending {} - {}",
            c_key.to_str().unwrap(),
            c_value.to_str().unwrap()
        );
        unsafe {
            apr_table_mergen(
                self.headers_out,
                apr_psprintf(
                    self.pool,
                    cstr!("%s"),
                    c_key.as_c_str().as_ptr(),
                ),
                apr_psprintf(
                    self.pool,
                    cstr!("%s"),
                    c_value.as_c_str().as_ptr()
                )
            );
        }
        Ok(())
    }

    fn set_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        let c_key = CString::new(key.as_str()).unwrap();
        let c_value = CString::new(value.to_str()?).unwrap();
        debug!(
            self.get_server_record().unwrap(),
            "request_rec::set_http_header - setting {} - {}",
            c_key.to_str().unwrap(),
            c_value.to_str().unwrap()
        );
        unsafe {
            apr_table_setn(
                self.headers_out,
                apr_psprintf(
                    self.pool,
                    cstr!("%s"),
                    c_key.as_c_str().as_ptr(),
                ),
                apr_psprintf(
                    self.pool,
                    cstr!("%s"),
                    c_value.as_c_str().as_ptr(),
                )
            );
        }
        Ok(())
    }

    fn set_content_encoding(
        &mut self,
        encoding: &ContentEncoding,
    ) -> () {
        match encoding {
            ContentEncoding::Gzip => self.content_encoding = cstr!("gzip"),
            _ => ()
        }
    }

    fn set_content_type(
        &mut self,
        mime: &Mime,
    ) -> () {
        let mime_str = CString::new(mime.essence_str()).unwrap();
        unsafe {
            ap_set_content_type(
                self as *mut request_rec,
                mime_str.as_c_str().as_ptr(),
            );
        }
    }

    fn set_content_length(
        &mut self,
        length: usize,
    ) -> () {
        unsafe {
            ap_set_content_length(
                self as *mut request_rec,
                length as i64,
            );
        }
    }

    fn write(
        &mut self,
        buffer: *const u8,
        length: usize,
    ) -> i32 {
        unsafe {
            ap_rwrite(
                buffer as *const c_void,
                length as i32,
                self
            )
        }
    }

    fn flush_response(&mut self) -> Result<(), ResponseWriteError> {
        let result = unsafe { ap_rflush(self as *mut request_rec) };
        if result < 0 {
            return Err(ResponseWriteError { error_code: result });
        } else {
            return Ok(())
        }
    }
}


#[cfg(test)]
impl HttpResponseWriter for request_rec {

    fn append_http_header(
        &mut self,
        _key: &HeaderName,
        _value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        Ok(())
    }

    fn set_http_header(
        &mut self,
        _key: &HeaderName,
        _value: &HeaderValue,
    ) -> Result<(), ToStrError> {
        Ok(())
    }

    fn set_content_encoding(
        &mut self,
        _encoding: &ContentEncoding,
    ) -> () {
    }

    fn set_content_type(
        &mut self,
        _mime: &Mime,
    ) -> () {
    }

    fn set_content_length(
        &mut self,
        _length: usize,
    ) -> () {
    }

    fn write(
        &mut self,
        _buffer: *const u8,
        length: usize,
    ) -> i32 {
        length as i32
    }

    fn flush_response(&mut self) -> Result<(), ResponseWriteError> {
        return Ok(())
    }
}


#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::io::communication::interface::HttpResponseWriter;
    use std::cmp::min;
    use std::collections::VecDeque;
    use std::slice;


    pub struct MockWriter {
        pub default_length: usize,
        pub allowed_lengths: VecDeque<usize>,
        pub written_payload: Vec<u8>,
        pub written_lengths: Vec<usize>,
    }

    impl MockWriter {
        pub fn new() -> MockWriter {
            MockWriter {
                default_length: 1,
                allowed_lengths: vec![2, 2, 2, 2].into_iter().collect(),
                written_payload: Vec::new(),
                written_lengths: Vec::new(),
            }
        }
    }

    impl HttpResponseWriter for MockWriter {

        fn append_http_header(
            &mut self,
            _key: &HeaderName,
            _value: &HeaderValue,
        ) -> Result<(), ToStrError> {
            Ok(())
        }

        fn set_http_header(
            &mut self,
            _key: &HeaderName,
            _value: &HeaderValue,
        ) -> Result<(), ToStrError> {
            Ok(())
        }

        fn set_content_encoding(
            &mut self,
            _encoding: &ContentEncoding,
        ) -> () {
        }

        fn set_content_type(
            &mut self,
            _mime: &Mime,
        ) -> () {
            ()
        }

        fn set_content_length(
            &mut self,
            _length: usize,
        ) -> () {
            ()
        }

        fn write(
            &mut self,
            payload: *const u8,
            length: usize,
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

        fn flush_response(&mut self) -> Result<(), ResponseWriteError> {
            Ok(())
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::MockWriter;
    use crate::framework::apache2::record::test_utils::with_request_rec;

    use std::error::Error as StdError;
    use std::ffi::CString;


    #[test]
    fn test_write_content_small_lengths() -> Result<(), Box<dyn StdError>> {
        let mut writer = MockWriter {
            default_length: 1,
            allowed_lengths: vec![2, 2, 2, 2].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            writer.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![2, 2, 2, 1], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }

    #[test]
    fn test_write_content_large_lengths() -> Result<(), Box<dyn StdError>> {
        let mut writer = MockWriter {
            default_length: 1,
            allowed_lengths: vec![128].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            writer.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![expected_payload.len()], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }

    #[test]
    fn test_write_content_paused_writes() -> Result<(), Box<dyn StdError>> {
        let mut writer = MockWriter {
            default_length: 1,
            allowed_lengths: vec![2, 0, 4, 0, 2].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            writer.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![2, 0, 4, 0, 1], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }

    #[test]
    fn test_write_content_delayed_writes() -> Result<(), Box<dyn StdError>> {
        let mut writer = MockWriter {
            default_length: 1,
            allowed_lengths: vec![0, 0, 4, 4].into_iter().collect(),
            written_payload: Vec::new(),
            written_lengths: Vec::new(),
        };
        let expected_payload = String::from("1234567");
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            writer.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![0, 0, 4, 3], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }
}