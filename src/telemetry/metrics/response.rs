use crate::apache2::response::ResponseContext;
use crate::interface::slippy::{ WriteResponseFunc, WriteResponseObserver, };
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult };
use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::slippy::result::{
    ReadRequestResult, ReadOutcome,
    WriteResponseResult, WriteOutcome,
};
use crate::schema::tile::config::MAX_ZOOM_SERVER;

use http::status::StatusCode;

use std::collections::hash_map::HashMap;
use std::vec::Vec;

pub struct ResponseAnalysis {
    response_count_by_status_and_zoom: HashMap<StatusCode, Vec<u32>>,
    tile_reponse_count_by_zoom: Vec<u32>,
}

impl ResponseAnalysis {
    pub fn new() -> ResponseAnalysis {
        ResponseAnalysis {
            response_count_by_status_and_zoom: HashMap::new(),
            tile_reponse_count_by_zoom: vec![0; MAX_ZOOM_SERVER + 1],
        }
    }

    fn on_handled_tile(
        &mut self,
        context: &ResponseContext,
        request: &request::Request,
        _response: &response::Response,
    ) -> () {
        let zoom_level = match &request.body {
            request::BodyVariant::ServeTileV2(v2_request) => v2_request.z as usize,
            request::BodyVariant::ServeTileV3(v3_request) => v3_request.z as usize,
            _ => { return; },
        };
        let zoom_limit = self.tile_reponse_count_by_zoom.len();
        if zoom_level < zoom_limit {
            self.tile_reponse_count_by_zoom[zoom_level] += 1;
        } else {
            warn!(
                context.get_host().record,
                "WriteResponseObserver::on_handled_tile - requested zoom level {} exceeds limit {}", zoom_level, zoom_limit
            );
        }
    }

    fn on_http_response_write(
        &mut self,
        context: &ResponseContext,
        request: &request::Request,
        http_response: &HttpResponse,
    ) -> () {
        if !(self.response_count_by_status_and_zoom.contains_key(&http_response.status_code)) {
            self.response_count_by_status_and_zoom.insert(http_response.status_code, vec![0; MAX_ZOOM_SERVER + 1]);
        }
        let count_by_zoom = self.response_count_by_status_and_zoom.get_mut(&http_response.status_code).unwrap();
        let zoom_level = match &request.body {
            request::BodyVariant::ServeTileV2(v2_request) => v2_request.z as usize,
            request::BodyVariant::ServeTileV3(v3_request) => v3_request.z as usize,
            _ => { return; },
        };
        let zoom_limit = count_by_zoom.len();
        if zoom_level < zoom_limit {
            count_by_zoom[zoom_level] += 1;
        } else {
            warn!(
                context.get_host().record,
                "WriteResponseObserver::on_http_response_write - requested zoom level {} exceeds limit {}", zoom_level, zoom_limit
            );
        }
    }
}

impl WriteResponseObserver for ResponseAnalysis {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        context: &ResponseContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
        write_result: &WriteResponseResult,
    ) -> () {
        match (read_result, handle_result) {
            (Ok(read_outcome), Ok(handle_outcome)) => match (read_outcome, handle_outcome) {
                (ReadOutcome::Matched(request), HandleOutcome::Handled(response)) => match response.body {
                    response::BodyVariant::Tile => self.on_handled_tile(context, request, response),
                    _ => (),
                },
                _ => (),
            },
            _ => ()
        }
        match (read_result, write_result) {
            (Ok(read_outcome), Ok(write_outcome)) => match (read_outcome, write_outcome) {
                (ReadOutcome::Matched(request), WriteOutcome::Written(http_response)) => self.on_http_response_write(context, request, http_response),
                _ => (),
            },
            _ => ()
        };
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::apache2::request::test_utils::with_request_rec;
    use crate::apache2::request::RequestContext;
    use crate::apache2::response::ResponseContext;
    use crate::schema::handler::result::HandleOutcome;
    use crate::schema::http::response::HttpResponse;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome };
    use crate::schema::tile::config::TileConfig;
    use http::header::HeaderMap;
    use http::status::StatusCode;
    use std::error::Error;
    use std::ffi::CString;

    fn mock_write(
        context: &mut ResponseContext,
        response: &response::Response
    ) -> WriteResponseResult {
        return Ok(WriteOutcome::NotWritten)
    }

    #[test]
    fn test_count_increment_on_server_tile_v3_write() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let tile_config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &tile_config)?;
            let read_result: ReadRequestResult = Ok(
                ReadOutcome::Matched(
                    request::Request {
                        header: request::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                        ),
                        body: request::BodyVariant::ServeTileV3(
                            request::ServeTileRequestV3 {
                                parameter: String::from("foo"),
                                x: 1,
                                y: 2,
                                z: 3,
                                extension: String::from("jpg"),
                                option: None,
                            }
                        ),
                    }
                )
            );
            let handle_result: HandleRequestResult = Ok(
                HandleOutcome::Handled(
                    response::Response {
                        header: response::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                            &mime::APPLICATION_JSON,
                        ),
                        body: response::BodyVariant::Tile,
                    }
                )
            );
            let write_result: WriteResponseResult = Ok(
                WriteOutcome::Written(
                    HttpResponse {
                        status_code: StatusCode::OK,
                        bytes_written: 8,
                        http_headers: HeaderMap::new(),
                    }
                )
            );
            let mut analysis = ResponseAnalysis::new();
            let response_context = ResponseContext::from(request_context);
            analysis.on_write(mock_write, &response_context, &read_result, &handle_result, &write_result);
            assert_eq!(
                1,
                analysis.response_count_by_status_and_zoom[&StatusCode::OK][3],
                "Response count not updated"
            );
            assert_eq!(
                0,
                analysis.response_count_by_status_and_zoom[&StatusCode::OK][2],
                "Response count does not default to 0"
            );
            assert_eq!(
                1,
                analysis.tile_reponse_count_by_zoom[3],
                "Tile count not updated"
            );
            assert_eq!(
                0,
                analysis.tile_reponse_count_by_zoom[2],
                "Tile count does not default to 0"
            );
            Ok(())
        })
    }

    #[test]
    fn test_count_increment_on_server_tile_v2_write() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let tile_config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &tile_config)?;
            let read_result: ReadRequestResult = Ok(
                ReadOutcome::Matched(
                    request::Request {
                        header: request::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                        ),
                        body: request::BodyVariant::ServeTileV2(
                            request::ServeTileRequestV2 {
                                x: 1,
                                y: 2,
                                z: 3,
                                extension: String::from("jpg"),
                                option: None,
                            }
                        ),
                    }
                )
            );
            let handle_result: HandleRequestResult = Ok(
                HandleOutcome::Handled(
                    response::Response {
                        header: response::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                            &mime::APPLICATION_JSON,
                        ),
                        body: response::BodyVariant::Tile,
                    }
                )
            );
            let write_result: WriteResponseResult = Ok(
                WriteOutcome::Written(
                    HttpResponse {
                        status_code: StatusCode::OK,
                        bytes_written: 8,
                        http_headers: HeaderMap::new(),
                    }
                )
            );
            let mut analysis = ResponseAnalysis::new();
            let response_context = ResponseContext::from(request_context);
            analysis.on_write(mock_write, &response_context, &read_result, &handle_result, &write_result);
            assert_eq!(
                1,
                analysis.response_count_by_status_and_zoom[&StatusCode::OK][3],
                "Response count not updated"
            );
            assert_eq!(
                0,
                analysis.response_count_by_status_and_zoom[&StatusCode::OK][2],
                "Response count does not default to 0"
            );
            assert_eq!(
                1,
                analysis.tile_reponse_count_by_zoom[3],
                "Tile count not updated"
            );
            assert_eq!(
                0,
                analysis.tile_reponse_count_by_zoom[2],
                "Tile count does not default to 0"
            );
            Ok(())
        })
    }

    #[test]
    fn test_no_increment_on_description_write() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let tile_config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &tile_config)?;
            let read_result: ReadRequestResult = Ok(
                ReadOutcome::Matched(
                    request::Request {
                        header: request::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                        ),
                        body: request::BodyVariant::DescribeLayer,
                    }
                )
            );
            let handle_result: HandleRequestResult = Ok(
                HandleOutcome::Handled(
                    response::Response {
                        header: response::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                            &mime::APPLICATION_JSON,
                        ),
                        body: response::BodyVariant::Description(
                            response::Description {
                                tilejson: "2.0.0",
                                schema: "xyz",
                                name: String::new(),
                                description: String::new(),
                                attribution: String::new(),
                                minzoom: 0,
                                maxzoom: 1,
                                tiles: Vec::new(),
                            }
                        ),
                    }
                )
            );
            let write_result: WriteResponseResult = Ok(
                WriteOutcome::Written(
                    HttpResponse {
                        status_code: StatusCode::OK,
                        bytes_written: 8,
                        http_headers: HeaderMap::new(),
                    }
                )
            );
            let mut analysis = ResponseAnalysis::new();
            let response_context = ResponseContext::from(request_context);
            analysis.on_write(mock_write, &response_context, &read_result, &handle_result, &write_result);
            let total_response_count = analysis.response_count_by_status_and_zoom[&StatusCode::OK].iter().fold(
                0,
                |acc, count| acc + count
            );
            let total_tile_count = analysis.tile_reponse_count_by_zoom.iter().fold(
                0,
                |acc, count| acc + count
            );
            assert_eq!(0, total_response_count, "Response count incremented on layer description response");
            assert_eq!(0, total_tile_count, "Tile response count incremented on layer description response");
            Ok(())
        })
    }

    #[test]
    fn test_reponse_count_increment_on_invalid_zoom_level() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let tile_config = TileConfig::new();
            let request_context = RequestContext::create_with_tile_config(request, &tile_config)?;
            let read_result: ReadRequestResult = Ok(
                ReadOutcome::Matched(
                    request::Request {
                        header: request::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                        ),
                        body: request::BodyVariant::ServeTileV2(
                            request::ServeTileRequestV2 {
                                x: 1,
                                y: 2,
                                z: (MAX_ZOOM_SERVER + 1) as u32,
                                extension: String::from("jpg"),
                                option: None,
                            }
                        ),
                    }
                )
            );
            let handle_result: HandleRequestResult = Ok(
                HandleOutcome::Handled(
                    response::Response {
                        header: response::Header::new(
                            request_context.record,
                            request_context.connection.record,
                            request_context.get_host().record,
                            &mime::APPLICATION_JSON,
                        ),
                        body: response::BodyVariant::Tile,
                    }
                )
            );
            let write_result: WriteResponseResult = Ok(
                WriteOutcome::Written(
                    HttpResponse {
                        status_code: StatusCode::OK,
                        bytes_written: 8,
                        http_headers: HeaderMap::new(),
                    }
                )
            );
            let mut analysis = ResponseAnalysis::new();
            let response_context = ResponseContext::from(request_context);
            analysis.on_write(mock_write, &response_context, &read_result, &handle_result, &write_result);
            assert!(
                analysis.response_count_by_status_and_zoom[&StatusCode::OK].get(MAX_ZOOM_SERVER + 1).is_none(),
                "A counter exists for invalid zoom level"
            );
            Ok(())
        })
    }
}
