use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::context::WriteContext;
use crate::schema::slippy::response::{ BodyVariant, Header, Description, SlippyResponse };
use crate::schema::slippy::result::{ WriteOutcome, WriteResponseResult };
use crate::apache2::request::RequestRecord;

use chrono::{ TimeZone, Utc, };
use http::header::{ CACHE_CONTROL, EXPIRES, ETAG, HeaderMap, HeaderValue };
use http::status::StatusCode;
use md5;
use mime;

pub struct SlippyResponseWriter { }
impl SlippyResponseWriter {
    pub fn write(
        context: &mut WriteContext,
        response: &SlippyResponse
    ) -> WriteResponseResult {
        match &response.body {
            BodyVariant::Description(descr) => {
                return DescriptionWriter::write(context, &response.header, descr);
            },
            BodyVariant::Tile(_) => {
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
        context: &mut WriteContext,
        header: &Header,
        description: &Description,
    ) -> WriteResponseResult {
        debug!(context.response.record.get_server_record().unwrap(), "DescriptionWriter::write - start");
        let mut http_headers = HeaderMap::new();
        let text = match (header.mime_type.type_(), header.mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => {
                context.response.set_content_type(&mime::APPLICATION_JSON);
                debug!(context.response.record.get_server_record().unwrap(), "DescriptionWriter::write - setting content type to {}", mime::APPLICATION_JSON.essence_str());
                serde_json::to_string_pretty(&description).unwrap()
            },
            _ => String::from(""),
        };
        let max_age: i64 = 7 * 24 * 60 * 60;

        let digest = format!("\"{:x}\"", md5::compute(&text));
        let etag_key = ETAG.clone();
        let etag_value = HeaderValue::from_str(digest.as_str()).unwrap();
        context.response.set_http_header(&etag_key, &etag_value).unwrap();
        http_headers.insert(etag_key, etag_value);


        let cache_age = format!("max-age={}", max_age);
        let cache_key = CACHE_CONTROL.clone();
        let cache_value = HeaderValue::from_str(cache_age.as_str()).unwrap();
        context.response.append_http_header(&cache_key, &cache_value).unwrap();
        http_headers.insert(cache_key, cache_value);

        let request_time_in_epoch_secs = context.response.record.request_time / 1000000;
        let expiry_in_epoch_secs = max_age + request_time_in_epoch_secs;
        let expiry_timestamp = Utc.timestamp(expiry_in_epoch_secs, 0);
        let expiry_string = expiry_timestamp.to_rfc2822();
        let expiry_key = EXPIRES.clone();
        let expiry_value = HeaderValue::from_str(expiry_string.as_str()).unwrap();
        context.response.set_http_header(&expiry_key, &expiry_value).unwrap();
        http_headers.insert(expiry_key, expiry_value);

        let written_length = context.response.write_content(&text)?;
        context.response.set_content_length(written_length);
        context.response.flush_response()?;
        debug!(context.response.record.get_server_record().unwrap(), "DescriptionWriter::write - finish");

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
