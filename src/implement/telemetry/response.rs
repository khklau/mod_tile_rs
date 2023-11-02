use crate::schema::apache2::config::{MAX_ZOOM_SERVER, ModuleConfig,};
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::slippy::result::{ReadOutcome, WriteOutcome,};
use crate::schema::tile::identity::LayerName;
use crate::interface::communication::HttpResponseWriter;
use crate::interface::slippy::{RequestContext, WriteResponseObserver,};
use crate::interface::telemetry::ResponseMetrics;

use chrono::Duration;
use http::status::StatusCode;

use std::boxed::Box;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::ops::Range;
use std::vec::Vec;


const VALID_ZOOM_RANGE: Range<u32> = 0..(MAX_ZOOM_SERVER as u32 + 1);

pub struct ResponseAnalysis {
    analysis_by_layer: HashMap<LayerName, LayerResponseAnalysis>,
    status_codes_responded: HashSet<StatusCode>,
}

impl ResponseAnalysis {
    pub fn new(_config: &ModuleConfig) -> Result<ResponseAnalysis, InvalidConfigError> {
        Ok(
            ResponseAnalysis {
                analysis_by_layer: HashMap::new(),
                status_codes_responded: HashSet::new(),
            }
        )
    }

    fn mut_layer<'s>(
        &'s mut self,
        request: &request::SlippyRequest,
    ) -> &'s mut LayerResponseAnalysis {
        let layer = &request.header.layer;
        self.analysis_by_layer.entry(layer.clone()).or_insert(LayerResponseAnalysis::new())
    }

    fn on_tile_write(
        &mut self,
        context: &RequestContext,
        request: &request::SlippyRequest,
        response: &response::SlippyResponse,
        response_duration: &Duration,
    ) -> () {
        self.accrue_tile_response_duration(context, request, response, response_duration);
        self.increment_tile_response_count(context, request, response);
    }

    fn accrue_tile_response_duration(
        &mut self,
        context: &RequestContext,
        request: &request::SlippyRequest,
        _response: &response::SlippyResponse,
        response_duration: &Duration,
    ) -> () {
        let zoom_level = match &request.body {
            request::BodyVariant::ServeTileV2(v2_request) => v2_request.z as usize,
            request::BodyVariant::ServeTileV3(v3_request) => v3_request.z as usize,
            _ => { return; },
        };
        let zoom_limit = self.mut_layer(request).tile_response_count_by_zoom.len();
        if zoom_level < zoom_limit {
            let counter = &mut(self.mut_layer(request).tile_response_duration_by_zoom[zoom_level]);
            *counter = *counter + *response_duration;
        } else {
            warn!(
                context.host.record,
                "ResponseAnalysis::accrue_tile_handle_duration - requested zoom level {} exceeds limit {}",
                zoom_level,
                zoom_limit,
            );
        }
    }

    fn increment_tile_response_count(
        &mut self,
        context: &RequestContext,
        request: &request::SlippyRequest,
        _response: &response::SlippyResponse,
    ) -> () {
        let zoom_level = match &request.body {
            request::BodyVariant::ServeTileV2(v2_request) => v2_request.z as usize,
            request::BodyVariant::ServeTileV3(v3_request) => v3_request.z as usize,
            _ => {
                return;
            },
        };
        let zoom_limit = self.mut_layer(request).tile_response_count_by_zoom.len();
        if zoom_level < zoom_limit {
            self.mut_layer(request).tile_response_count_by_zoom[zoom_level] += 1;
        } else {
            warn!(
                context.host.record,
                "ResponseAnalysis::on_tile_write - requested zoom level {} exceeds limit {}", zoom_level, zoom_limit
            );
        }
    }

    fn on_http_response_write(
        &mut self,
        context: &RequestContext,
        request: &request::SlippyRequest,
        http_response: &HttpResponse,
    ) -> () {
        self.mut_layer(request)
            .response_count_by_status_and_zoom.entry(http_response.status_code.clone())
            .or_insert(vec![0; MAX_ZOOM_SERVER + 1]);
        self.status_codes_responded.insert(http_response.status_code.clone());
        let count_by_zoom = self.mut_layer(request).response_count_by_status_and_zoom.get_mut(
            &http_response.status_code
        ).unwrap();
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
                context.host.record,
                "ResponseAnalysis::on_http_response_write - requested zoom level {} exceeds limit {}", zoom_level, zoom_limit
            );
        }
    }
}

struct LayerResponseAnalysis {
    response_count_by_status_and_zoom: HashMap<StatusCode, Vec<u64>>,
    tile_response_count_by_zoom: Vec<u64>,
    tile_response_duration_by_zoom: Vec<Duration>,
}

impl LayerResponseAnalysis {
    pub fn new() -> LayerResponseAnalysis {
        LayerResponseAnalysis {
            response_count_by_status_and_zoom: HashMap::new(),
            tile_response_count_by_zoom: vec![0; VALID_ZOOM_RANGE.end as usize],
            tile_response_duration_by_zoom: vec![Duration::zero(); VALID_ZOOM_RANGE.end as usize],
        }
    }
}

impl WriteResponseObserver for ResponseAnalysis {
    fn on_write(
        &mut self,
        context: &RequestContext,
        _response: &response::SlippyResponse,
        _writer: &dyn HttpResponseWriter,
        write_outcome: &WriteOutcome,
        _write_func_name: &'static str,
        read_outcome: &ReadOutcome,
        handle_outcome: &HandleOutcome,
    ) -> () {
        match (&read_outcome, &handle_outcome) {
            (ReadOutcome::Processed(read_result), HandleOutcome::Processed(handle_result)) => {
                let response_duration = handle_result.after_timestamp - handle_result.before_timestamp; // FIXME does not include read duration
                match (read_result, &handle_result.result) {
                    (Ok(request), Ok(response)) => match response.body {
                        response::BodyVariant::Tile(_) => self.on_tile_write(context, request, response, &response_duration),
                        _ => (),
                    },
                    _ => (),
                }
            },
            _ => ()
        }
        match (read_outcome, write_outcome) {
            (ReadOutcome::Processed(read_result), WriteOutcome::Processed(write_outcome)) => match (read_result, write_outcome) {
                (Ok(request), Ok(http_response)) => self.on_http_response_write(context, request, http_response),
                _ => (),
            },
            _ => ()
        };
    }
}

impl ResponseMetrics for ResponseAnalysis {
    fn iterate_status_codes_responded(&self) -> Vec<StatusCode> {
        self.status_codes_responded.iter().cloned().collect()
    }

    fn iterate_valid_zoom_levels(&self) -> Range<u32> {
        VALID_ZOOM_RANGE.clone()
    }

    fn iterate_layers_responded(&self) -> Vec<LayerName> {
        self.analysis_by_layer.keys().cloned().collect()
    }

    fn count_response_by_status_code(&self, status_code: &StatusCode) -> u64 {
        let mut total = 0;
        for layer_analysis in self.analysis_by_layer.values() {
            if layer_analysis.response_count_by_status_and_zoom.contains_key(status_code) {
                let layer_count: u64 = layer_analysis.response_count_by_status_and_zoom[status_code].iter().sum();
                total += layer_count;
            }
        }
        return total;
    }

    fn count_response_by_zoom_level(&self, zoom: u32) -> u64 {
        let mut total = 0;
        for layer_analysis in self.analysis_by_layer.values() {
            for counts_by_zoom in layer_analysis.response_count_by_status_and_zoom.values() {
                if counts_by_zoom.len() > (zoom as usize) {
                    total += counts_by_zoom[zoom as usize];
                }
            }
        }
        return total;
    }

    fn count_response_by_status_code_and_zoom_level(&self, status_code: &StatusCode, zoom: u32) -> u64 {
        let mut total = 0;
        for layer_analysis in self.analysis_by_layer.values() {
            if layer_analysis.response_count_by_status_and_zoom.contains_key(status_code) {
                let counts_by_zoom = &(layer_analysis.response_count_by_status_and_zoom[status_code]);
                if counts_by_zoom.len() > (zoom as usize) {
                    total += counts_by_zoom[zoom as usize];
                }
            }
        }
        return total;
    }

    fn count_total_tile_response(&self) -> u64 {
        let mut total = 0;
        for layer_analysis in self.analysis_by_layer.values() {
            let count: u64 = layer_analysis.tile_response_count_by_zoom.iter().sum();
            total += count;
        }
        return total;
    }

    fn tally_total_tile_response_duration(&self) -> u64 {
        let mut total_duration = Duration::zero();
        for layer_analysis in self.analysis_by_layer.values() {
            let duration = layer_analysis.tile_response_duration_by_zoom.iter().fold(
                Duration::zero(),
                |acc, duration| acc + *duration
            );
            total_duration = total_duration + duration;
        }
        return total_duration.num_seconds() as u64
    }

    fn count_tile_response_by_zoom_level(&self, zoom: u32) -> u64 {
        let mut total = 0;
        for layer_analysis in self.analysis_by_layer.values() {
            if layer_analysis.tile_response_count_by_zoom.len() > (zoom as usize) {
                total += layer_analysis.tile_response_count_by_zoom[zoom as usize]
            }
        }
        return total;
    }

    fn tally_tile_response_duration_by_zoom_level(&self, zoom: u32) -> u64 {
        let mut total = 0;
        for layer_analysis in self.analysis_by_layer.values() {
            if layer_analysis.tile_response_duration_by_zoom.len() > (zoom as usize) {
                total += layer_analysis.tile_response_duration_by_zoom[zoom as usize].num_seconds() as u64
            }
        }
        return total;
    }

    fn count_response_by_layer_and_status_code(&self, layer: &LayerName, status_code: &StatusCode) -> u64 {
        match self.analysis_by_layer.get(layer) {
            Some(layer_analysis) => {
                if layer_analysis.response_count_by_status_and_zoom.contains_key(status_code) {
                    layer_analysis.response_count_by_status_and_zoom[status_code].iter().sum()
                } else {
                    0
                }
            },
            None => 0
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::apache2::connection::Connection;
    use crate::schema::apache2::request::Apache2Request;
    use crate::schema::apache2::virtual_host::VirtualHost;
    use crate::schema::handler::result::HandleRequestResult;
    use crate::schema::http::encoding::ContentEncoding;
    use crate::schema::http::response::HttpResponse;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::schema::slippy::result::WriteOutcome;
    use crate::schema::tile::age::TileAge;
    use crate::schema::tile::source::TileSource;
    use crate::interface::apache2::PoolStored;
    use crate::interface::tile::TileRef;
    use crate::framework::apache2::record::test_utils::with_request_rec;
    use crate::implement::communication::http_exchange::test_utils::MockWriter;
    use chrono::Utc;
    use http::header::HeaderMap;
    use http::status::StatusCode;
    use std::error::Error;
    use std::ffi::CString;
    use std::rc::Rc;

    #[test]
    fn test_count_increment_on_tile_render() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let module_config = ModuleConfig::new();
            let context = RequestContext {
                module_config: &module_config,
                host: VirtualHost::find_or_allocate_new(request)?,
                request: Apache2Request::create_with_tile_config(request)?,
            };
            let layer_name = LayerName::from("default");
            let read_outcome = ReadOutcome::Processed(
                Ok(
                    request::SlippyRequest {
                        header: request::Header::new_with_layer(
                            context.request.record,
                            &layer_name,
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
            let before_timestamp = Utc::now();
            let response_duration = Duration::seconds(2);
            let after_timestamp = before_timestamp + response_duration;
            let empty_tile: Rc<Vec<u8>> = Rc::new(Vec::new());
            let tile_ref = TileRef {
                raw_bytes: Rc::downgrade(&empty_tile),
                begin: 0,
                end: 1,
                media_type: mime::IMAGE_PNG,
                encoding: ContentEncoding::NotCompressed,
            };
            let response = response::SlippyResponse {
                header: response::Header::new(
                    context.request.record,
                    &mime::APPLICATION_JSON,
                ),
                body: response::BodyVariant::Tile(
                    response::TileResponse {
                        source: TileSource::Render,
                        age: TileAge::Fresh,
                        tile_ref,
                    }
                ),
            };
            let handle_outcome = HandleOutcome::Processed(
                HandleRequestResult {
                    before_timestamp,
                    after_timestamp,
                    result: Ok(response.clone()),
                }
            );
            let write_outcome = WriteOutcome::Processed(
                Ok(
                    HttpResponse {
                        status_code: StatusCode::OK,
                        bytes_written: 508,
                        http_headers: HeaderMap::new(),
                    }
                )
            );
            let mut analysis = ResponseAnalysis::new(&module_config)?;
            let writer = MockWriter::new();
            analysis.on_write(&context, &response, &writer, &write_outcome, "mock", &read_outcome, &handle_outcome);
            assert_eq!(
                0,
                analysis.count_response_by_status_code_and_zoom_level(&StatusCode::OK, 5),
                "Response count does not default to 0"
            );
            assert_eq!(
                0,
                analysis.count_response_by_status_code_and_zoom_level(&StatusCode::BAD_REQUEST, 3),
                "Response count does not default to 0"
            );
            assert_eq!(
                1,
                analysis.count_response_by_status_code_and_zoom_level(&StatusCode::OK, 3),
                "Response count not incremented"
            );
            assert_eq!(
                0,
                analysis.count_tile_response_by_zoom_level(99),
                "Tile response count not incremented"
            );
            assert_eq!(
                1,
                analysis.count_tile_response_by_zoom_level(3),
                "Tile response count not incremented"
            );
            assert_eq!(
                0,
                analysis.tally_tile_response_duration_by_zoom_level(99),
                "Tile response duration not tallied"
            );
            assert_eq!(
                response_duration.num_seconds() as u64,
                analysis.tally_tile_response_duration_by_zoom_level(3),
                "Tile response duration not tallied"
            );
            assert_eq!(
                0,
                analysis.count_response_by_layer_and_status_code(&LayerName::from("foobar"), &StatusCode::OK),
                "Response count not incremented"
            );
            assert_eq!(
                0,
                analysis.count_response_by_layer_and_status_code(&layer_name, &StatusCode::BAD_REQUEST),
                "Response count not incremented"
            );
            assert_eq!(
                1,
                analysis.count_response_by_layer_and_status_code(&layer_name, &StatusCode::OK),
                "Response count not incremented"
            );
            Ok(())
        })
    }
}
