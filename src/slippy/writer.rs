use crate::slippy::interface::{ WriteOutcome, WriteResponseResult };
use crate::slippy::response::{ BodyVariant, Header, Description, Response };

use crate::apache2::response::{ HttpResponse, ResponseContext, };

use chrono::{ TimeZone, Utc, };
use http::header::{ CACHE_CONTROL, EXPIRES, ETAG, HeaderMap, HeaderValue };
use http::status::StatusCode;
use md5;
use mime;

pub struct SlippyResponseWriter { }
impl SlippyResponseWriter {
    pub fn write(
        context: &mut ResponseContext,
        response: &Response
    ) -> WriteResponseResult {
        match &response.body {
            BodyVariant::Description(descr) => {
                return DescriptionWriter::write(context, &response.header, descr);
            },
            BodyVariant::Tile => {
                return Ok(
                    WriteOutcome::NotWritten
                )
            },
        }
    }
}

struct DescriptionWriter { }
impl DescriptionWriter {
    pub fn write(
        context: &mut ResponseContext,
        header: &Header,
        description: &Description,
    ) -> WriteResponseResult {
        let mut http_headers = HeaderMap::new();
        let text = match (header.mime_type.type_(), header.mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => {
                context.set_content_type(&mime::APPLICATION_JSON);
                serde_json::to_string(&description).unwrap()
            },
            _ => String::from(""),
        };
        let written_length = context.write_content(&text)?;
        context.set_content_length(written_length);
        let max_age: i64 = 7 * 24 * 60 * 60;

        let digest = format!("\"{:x}\"", md5::compute(&text));
        let etag_key = ETAG.clone();
        let etag_value = HeaderValue::from_str(digest.as_str()).unwrap();
        context.set_http_header(&etag_key, &etag_value).unwrap();
        http_headers.insert(etag_key, etag_value);


        let cache_age = format!("max-age={}", max_age);
        let cache_key = CACHE_CONTROL.clone();
        let cache_value = HeaderValue::from_str(cache_age.as_str()).unwrap();
        context.append_http_header(&cache_key, &cache_value).unwrap();
        http_headers.insert(cache_key, cache_value);

        let expiry_timestamp = Utc.timestamp(max_age + context.request_record.request_time, 0);
        let expiry_string = expiry_timestamp.to_rfc2822();
        let expiry_key = EXPIRES.clone();
        let expiry_value = HeaderValue::from_str(expiry_string.as_str()).unwrap();
        context.set_http_header(&expiry_key, &expiry_value).unwrap();
        http_headers.insert(expiry_key, expiry_value);

        Ok(
            WriteOutcome::Written(
                HttpResponse {
                    status_code: StatusCode::OK,
                    bytes_written: written_length,
                    http_headers,
                }
            )
        )
    }
}
