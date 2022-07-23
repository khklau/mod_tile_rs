use crate::binding::apache2::request_rec;
use crate::schema::apache2::error::ResponseWriteError;
use crate::schema::http::encoding::ContentEncoding;

use http::header::{ HeaderName, HeaderValue, ToStrError, };
use mime::Mime;

use std::boxed::Box;
use std::error::Error;
use std::option::Option;
use std::result::Result;

use std::ffi::CString;
use std::mem::size_of;


pub trait PoolStored<'p> {
    fn search_pool_key(request: &request_rec) -> CString;

    fn find(request: &'p request_rec, pool_key: &CString) -> Result<Option<&'p mut Self>, Box<dyn Error>>;

    fn new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>>;

    fn find_or_allocate_new(request: &'p request_rec) -> Result<&'p mut Self, Box<dyn Error>> {
        let id = Self::search_pool_key(request);
        match Self::find(request, &id)? {
            Some(existing) => Ok(existing),
            None => Ok(Self::new(request)?),
        }
    }
}

pub trait Writer {

    fn append_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError>;

    fn set_http_header(
        &mut self,
        key: &HeaderName,
        value: &HeaderValue,
    ) -> Result<(), ToStrError>;

    fn set_content_encoding(
        &mut self,
        encoding: &ContentEncoding,
    ) -> ();

    fn set_content_type(
        &mut self,
        mime: &Mime,
    ) -> ();

    fn set_content_length(
        &mut self,
        length: usize,
    ) -> ();

    fn write_content(
        &mut self,
        payload: &dyn AsRef<[u8]>,
    ) -> Result<usize, ResponseWriteError> {
        let mut payload_slice = payload.as_ref();
        while payload_slice.len() > 0 {
            let result = self.write(
                payload_slice.as_ptr(),
                payload_slice.len(),
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

    fn write(
        &mut self,
        buffer: *const u8,
        length: usize,
    ) -> i32;

    fn flush_response(&mut self) -> Result<(), ResponseWriteError>;
}
