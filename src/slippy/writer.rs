use crate::slippy::interface::{ WriteOutcome, WriteResponseResult };
use crate::slippy::response::{ BodyVariant, Header, Description, Response };

use crate::apache2::response::{ HttpResponse, ResponseContext };

use http::status::StatusCode;
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
        let text = match (header.mime_type.type_(), header.mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => serde_json::to_string(&description).unwrap(),
            _ => String::from(""),
        };
        Ok(
            WriteOutcome::Written(
                HttpResponse {
                    status_code: StatusCode::OK,
                    bytes_written: 0,
                    http_header: None,
                }
            )
        )
    }
}
