use crate::binding::apache2::request_rec;
use crate::schema::http::request::HttpRequest;

use chrono::{TimeZone, Utc,};

use std::ffi::CStr;
use std::result::Result;
use std::str::Utf8Error;


pub fn read_apache2_request(request: & request_rec) -> Result<HttpRequest, Utf8Error> {
    let uri = unsafe {
        CStr::from_ptr(request.uri)
    }.to_str()?;
    let received_timestamp = Utc.timestamp_millis(request.request_time);
    Ok(
        HttpRequest::new(
            uri,
            received_timestamp,
            request,
        )
    )
}
