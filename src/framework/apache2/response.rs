
use crate::binding::apache2::request_rec;
#[cfg(not(test))]
use crate::binding::apache2::{
    ap_rwrite, ap_rflush, ap_set_content_type, ap_set_content_length,
    apr_psprintf, apr_table_setn, apr_table_mergen,
};
use crate::schema::apache2::error::ResponseWriteError;
use crate::framework::apache2::record::RequestRecord;

use http::header::{ HeaderName, HeaderValue, ToStrError, };
use mime::Mime;

#[cfg(not(test))]
use std::ffi::{ CString, c_void };
use std::mem::size_of;
use std::option::Option;


trait Writer {
    type ElementType;

    fn write(
        &mut self,
        buffer: *const Self::ElementType,
        length: usize,
    ) -> i32;
}

pub struct Apache2Response<'r> {
    pub record: &'r mut request_rec,
    writer: Option<&'r mut dyn Writer<ElementType = u8>>,
}

impl<'r> Apache2Response<'r> {
    pub fn from(record: &'r mut request_rec) -> Apache2Response<'r> {
        Apache2Response {
            record,
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
        debug!(
            self.record.get_server_record().unwrap(),
            "Response::append_http_header - appending {} - {}",
            c_key.to_str().unwrap(),
            c_value.to_str().unwrap()
        );
        unsafe {
            apr_table_mergen(
                self.record.headers_out,
                apr_psprintf(
                    self.record.pool,
                    cstr!("%s"),
                    c_key.as_c_str().as_ptr(),
                ),
                apr_psprintf(
                    self.record.pool,
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
            self.record.get_server_record().unwrap(),
            "Response::set_http_header - setting {} - {}",
            c_key.to_str().unwrap(),
            c_value.to_str().unwrap()
        );
        unsafe {
            apr_table_setn(
                self.record.headers_out,
                apr_psprintf(
                    self.record.pool,
                    cstr!("%s"),
                    c_key.as_c_str().as_ptr(),
                ),
                apr_psprintf(
                    self.record.pool,
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
                self.record as *mut request_rec,
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
                self.record as *mut request_rec,
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
            Some(obj) => *obj,
            None => {
                // Work around the borrow checker below, but its necessary since request_rec from a foreign C framework
                let write_record = self.record as *mut request_rec;
                unsafe { write_record.as_mut().unwrap() }
            }
        };
        let mut payload_slice = payload.as_ref();
        while payload_slice.len() > 0 {
            debug!(
                self.record.get_server_record().unwrap(),
                "Response::write_content - writing slice {}",
                String::from_utf8_lossy(payload_slice)
            );
            let result = writer.write(
                payload_slice.as_ptr(),
                payload_slice.len(),
            );
            debug!(
                self.record.get_server_record().unwrap(),
                "Response::write_content - write result {}",
                result
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

    #[cfg(not(test))]
    pub fn flush_response(&mut self) -> Result<(), ResponseWriteError> {
        let result = unsafe { ap_rflush(self.record as *mut request_rec) };
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


impl Writer for request_rec {
    type ElementType = u8;

    #[cfg(not(test))]
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

    #[cfg(test)]
    fn write(
        &mut self,
        _buffer: *const u8,
        length: usize,
    ) -> i32 {
        length as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::apache2::record::test_utils::with_request_rec;

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

    impl Writer for TestWriter {
        type ElementType = u8;

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
            let mut context = Apache2Response::from(request);
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
            let mut context = Apache2Response::from(request);
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
            let mut context = Apache2Response::from(request);
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
            let mut context = Apache2Response::from(request);
            context.writer = Some(&mut writer);
            context.write_content(&expected_payload)?;
            assert_eq!((&expected_payload).as_ref(), writer.written_payload, "Unexpected payload written");
            assert_eq!(vec![0, 0, 4, 3], writer.written_lengths, "Unexpected written lengths");
            Ok(())
        })
    }
}