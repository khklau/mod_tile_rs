use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::error::WriteError;
use crate::schema::slippy::response::{
    BodyVariant, Header, Description, SlippyResponse, Statistics, TileResponse,
};
use crate::schema::slippy::result::{ WriteOutcome, WriteResponseResult, };
use crate::interface::apache2::Writer;
use crate::interface::slippy::WriteContext;

use chrono::{ TimeZone, Utc, };
use http::header::{ CACHE_CONTROL, EXPIRES, ETAG, HeaderMap, HeaderValue };
use http::status::StatusCode;
use md5;
use mime;

pub struct SlippyResponseWriter { }
impl SlippyResponseWriter {
    pub fn write(
        context: &WriteContext,
        response: &SlippyResponse,
        writer: &mut dyn Writer,
    ) -> WriteOutcome {
        match &response.body {
            BodyVariant::Description(description) => {
                WriteOutcome::Processed(
                    DescriptionWriter::write(context, &response.header, description, writer)
                )
            },
            BodyVariant::Statistics(statistics) => {
                WriteOutcome::Processed(
                    StatisticsWriter::write(context, &response.header, statistics, writer)
                )
            },
            BodyVariant::Tile(tile) => {
                WriteOutcome::Processed(
                    TileWriter::write(context, &response.header, tile, writer)
                )
            },
        }
    }
}

struct DescriptionWriter { }
impl DescriptionWriter {
    pub fn write(
        context: &WriteContext,
        header: &Header,
        description: &Description,
        writer: &mut dyn Writer,
    ) -> WriteResponseResult {
        debug!(context.host.record, "DescriptionWriter::write - start");
        let mut http_headers = HeaderMap::new();
        let text = match (header.mime_type.type_(), header.mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => {
                writer.set_content_type(&mime::APPLICATION_JSON);
                debug!(context.host.record, "DescriptionWriter::write - setting content type to {}", mime::APPLICATION_JSON.essence_str());
                serde_json::to_string_pretty(description).unwrap()
            },
            _ => String::from(""),
        };
        let max_age: i64 = 7 * 24 * 60 * 60;

        let digest = format!("\"{:x}\"", md5::compute(&text));
        let etag_key = ETAG.clone();
        let etag_value = HeaderValue::from_str(digest.as_str()).unwrap();
        writer.set_http_header(&etag_key, &etag_value).unwrap();
        http_headers.insert(etag_key, etag_value);


        let cache_age = format!("max-age={}", max_age);
        let cache_key = CACHE_CONTROL.clone();
        let cache_value = HeaderValue::from_str(cache_age.as_str()).unwrap();
        writer.append_http_header(&cache_key, &cache_value).unwrap();
        http_headers.insert(cache_key, cache_value);

        let request_time_in_epoch_secs = context.request.record.request_time / 1000000;
        let expiry_in_epoch_secs = max_age + request_time_in_epoch_secs;
        let expiry_timestamp = Utc.timestamp(expiry_in_epoch_secs, 0);
        let expiry_string = expiry_timestamp.to_rfc2822();
        let expiry_key = EXPIRES.clone();
        let expiry_value = HeaderValue::from_str(expiry_string.as_str()).unwrap();
        writer.set_http_header(&expiry_key, &expiry_value).unwrap();
        http_headers.insert(expiry_key, expiry_value);

        let written_length = writer.write_content(&text)?;
        writer.set_content_length(written_length);
        writer.flush_response()?;
        debug!(context.host.record, "DescriptionWriter::write - finish");

        Ok(
            HttpResponse {
                status_code: StatusCode::OK,
                bytes_written: written_length,
                http_headers,
            }
        )
    }
}

struct StatisticsWriter { }
impl StatisticsWriter {
    pub fn write(
        context: &WriteContext,
        header: &Header,
        statistics: &Statistics,
        writer: &mut dyn Writer,
    ) -> WriteResponseResult {
        debug!(context.host.record, "StatisticsWriter::write - start");
        let mut http_headers = HeaderMap::new();
        let text = match (header.mime_type.type_(), header.mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => {
                writer.set_content_type(&mime::APPLICATION_JSON);
                debug!(context.host.record, "StatisticsWriter::write - setting content type to {}", mime::APPLICATION_JSON.essence_str());
                serde_json::to_string_pretty(statistics).unwrap()
            },
            _ => String::from(""),
        };

        let digest = format!("\"{:x}\"", md5::compute(&text));
        let etag_key = ETAG.clone();
        let etag_value = HeaderValue::from_str(digest.as_str()).unwrap();
        writer.set_http_header(&etag_key, &etag_value).unwrap();
        http_headers.insert(etag_key, etag_value);

        let written_length = writer.write_content(&text)?;
        writer.set_content_length(written_length);
        writer.flush_response()?;
        debug!(context.host.record, "StatisticsWriter::write - finish");

        Ok(
            HttpResponse {
                status_code: StatusCode::OK,
                bytes_written: written_length,
                http_headers,
            }
        )
    }
}

struct TileWriter {}
impl TileWriter {
    pub fn write(
        context: &WriteContext,
        header: &Header,
        tile: &TileResponse,
        writer: &mut dyn Writer,
    ) -> WriteResponseResult {
        debug!(context.host.record, "TileWriter::write - start");
        let result = if let (mime::IMAGE, mime::PNG) = (header.mime_type.type_(), header.mime_type.subtype()) {
            let mut http_headers = HeaderMap::new();
            writer.set_content_type(&mime::IMAGE_PNG);
            debug!(context.host.record, "TileWriter::write - setting content type to {}", mime::IMAGE_PNG.essence_str());
            tile.tile_ref.with_tile(|raw_bytes| {
                let digest = format!("\"{:x}\"", md5::compute(&raw_bytes));
                let etag_key = ETAG.clone();
                let etag_value = HeaderValue::from_str(digest.as_str()).unwrap();
                writer.set_http_header(&etag_key, &etag_value).unwrap();
                http_headers.insert(etag_key, etag_value);
                let written_length = writer.write_content(&raw_bytes)?;
                writer.set_content_length(written_length);
                writer.flush_response()?;
                Ok(
                    HttpResponse {
                        status_code: StatusCode::OK,
                        bytes_written: written_length,
                        http_headers,
                    }
                )
            })
        } else {
            Err(
                WriteError::UnsupportedMediaType(tile.tile_ref.media_type.clone())
            )
        };
        debug!(context.host.record, "TileWriter::write - finish");
        return result;
    }
}
