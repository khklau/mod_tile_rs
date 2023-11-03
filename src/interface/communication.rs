use crate::schema::communication::error::ResponseWriteError;
use crate::schema::http::encoding::ContentEncoding;
use crate::interface::context::RequestContext;

use http::header::{ HeaderName, HeaderValue, ToStrError, };
use mime::Mime;

use std::mem::size_of;
use std::option::Option;
use std::result::Result;
use std::string::String;


#[derive(Debug)]
pub enum CommunicationError {
    TimeoutError,
    Io(std::io::Error),
}

impl From<std::io::Error> for CommunicationError {
    fn from(error: std::io::Error) -> Self {
        return CommunicationError::Io(error);
    }
}

pub enum RenderResponse {
    NotDone,
    Done(String),
}

pub trait BidirectionalChannel {
    fn send_blocking_request(
        &mut self,
        context: &RequestContext,
        request: &[u8],
        response_buffer: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, CommunicationError>;
}

pub trait HttpResponseWriter {

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

pub struct RenderdCommunicationInventory<'i> {
    pub primary_comms: &'i mut dyn BidirectionalChannel,
}

pub trait CommunicationInventory {
    fn primary_renderd_comms(&mut self) -> &mut dyn BidirectionalChannel;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub struct EmptyResultBiChannel { }

    impl BidirectionalChannel for EmptyResultBiChannel {
        fn send_blocking_request(
            &mut self,
            _context: &RequestContext,
            _request: &[u8],
            _response_buffer: Option<Vec<u8>>,
        ) -> Result<Vec<u8>, CommunicationError> {
            Ok(Vec::new())
        }
    }

    pub struct EmptyResultCommunicationInventory {
        renderd_comms: EmptyResultBiChannel,
    }

    impl EmptyResultCommunicationInventory {
        pub fn new() -> EmptyResultCommunicationInventory {
            EmptyResultCommunicationInventory {
                renderd_comms: EmptyResultBiChannel {  }
            }
        }
    }

    impl CommunicationInventory for EmptyResultCommunicationInventory {
        fn primary_renderd_comms(&mut self) -> &mut dyn BidirectionalChannel {
            &mut self.renderd_comms
        }
    }

}
