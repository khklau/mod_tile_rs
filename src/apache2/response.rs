use crate::apache2::bindings::request_rec;
use crate::apache2::request::RequestContext;

use http::header::{ HeaderName, HeaderValue };
use http::status::StatusCode;

use std::option::Option;


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
}
