use crate::schema::apache2::config::MAX_ZOOM_SERVER;
use crate::schema::handler::context::HandleContext;
use crate::schema::slippy::context::WriteContext;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler };
use crate::interface::slippy::{ WriteResponseFunc, WriteResponseObserver, };
use crate::interface::telemetry::metrics::{
    CacheMetrics, RenderMetrics, ResponseMetrics
};
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult };
use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::slippy::result::{
    ReadRequestResult, ReadOutcome,
    WriteResponseResult, WriteOutcome,
};
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::apache2::request::RequestRecord;

use chrono::Duration;
use http::status::StatusCode;

use std::collections::hash_map::HashMap;
use std::slice::Iter;
use std::vec::Vec;
use core::default::Default;


pub struct ResponseAnalysis {
    response_count_by_status_and_zoom: HashMap<StatusCode, Vec<u64>>,
    tile_reponse_count_by_zoom: Vec<u64>,
    tile_handle_duration_by_zoom: Vec<Duration>,
    tile_handle_count_by_source_and_age: TileMetricTable<u64>,
}

impl ResponseAnalysis {
    pub fn new() -> ResponseAnalysis {
        ResponseAnalysis {
            response_count_by_status_and_zoom: HashMap::new(),
            tile_reponse_count_by_zoom: vec![0; MAX_ZOOM_SERVER + 1],
            tile_handle_duration_by_zoom: vec![Duration::zero(); MAX_ZOOM_SERVER + 1],
            tile_handle_count_by_source_and_age: TileMetricTable::new(),
        }
    }

    fn on_handled_tile(
        &mut self,
        context: &HandleContext,
        request: &request::SlippyRequest,
        response: &response::TileResponse,
        handle_duration: &Duration,
    ) -> () {
        self.accrue_tile_handle_duration(context, request, handle_duration);
        self.increment_tile_handle_count(context, response);
    }

    fn accrue_tile_handle_duration(
        &mut self,
        context: &HandleContext,
        request: &request::SlippyRequest,
        handle_duration: &Duration,
    ) -> () {
        let zoom_level = match &request.body {
            request::BodyVariant::ServeTileV2(v2_request) => v2_request.z as usize,
            request::BodyVariant::ServeTileV3(v3_request) => v3_request.z as usize,
            _ => { return; },
        };
        let zoom_limit = self.tile_reponse_count_by_zoom.len();
        if zoom_level < zoom_limit {
            self.tile_handle_duration_by_zoom[zoom_level] = self.tile_handle_duration_by_zoom[zoom_level] + *handle_duration;
        } else {
            warn!(
                context.host.record,
                "ResponseAnalysis::accrue_tile_handle_duration - requested zoom level {} exceeds limit {}", zoom_level, zoom_limit
            );
        }
    }

    fn increment_tile_handle_count(
        &mut self,
        context: &HandleContext,
        response: &response::TileResponse,
    ) -> () {
        let counter = self.tile_handle_count_by_source_and_age.update(&response.source, &response.age);
        *counter += 1;
        debug!(
            context.host.record,
            "ResponseAnalysis::increment_tile_handle_count - updating count for source {:?} and age {:?} to {}",
            &response.source,
            &response.age,
            counter,
        );
    }

    fn on_tile_write(
        &mut self,
        context: &WriteContext,
        request: &request::SlippyRequest,
        _response: &response::SlippyResponse,
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
                context.response.record.get_server_record().unwrap(),
                "ResponseAnalysis::on_tile_write - requested zoom level {} exceeds limit {}", zoom_level, zoom_limit
            );
        }
    }

    fn on_http_response_write(
        &mut self,
        context: &WriteContext,
        request: &request::SlippyRequest,
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
                context.response.record.get_server_record().unwrap(),
                "ResponseAnalysis::on_http_response_write - requested zoom level {} exceeds limit {}", zoom_level, zoom_limit
            );
        }
    }
}

impl WriteResponseObserver for ResponseAnalysis {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        context: &WriteContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
        write_result: &WriteResponseResult,
    ) -> () {
        match (read_result, &handle_result.result) {
            (Ok(read_outcome), Ok(handle_outcome)) => match (read_outcome, handle_outcome) {
                (ReadOutcome::Matched(request), HandleOutcome::Handled(response)) => match response.body {
                    response::BodyVariant::Tile(_) => self.on_tile_write(context, request, response),
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

impl HandleRequestObserver for ResponseAnalysis {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        context: &HandleContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
    ) -> () {
        let handle_duration = handle_result.after_timestamp - handle_result.before_timestamp;
        match (read_result, &handle_result.result) {
            (Ok(read_outcome), Ok(handle_outcome)) => match (read_outcome, handle_outcome) {
                (ReadOutcome::Matched(request), HandleOutcome::Handled(response)) => match &response.body {
                    response::BodyVariant::Tile(tile_response) => self.on_handled_tile(context, request, &tile_response, &handle_duration),
                    _ => (),
                },
                _ => (),
            },
            _ => ()
        }
    }
}

impl ResponseMetrics for ResponseAnalysis {
    fn count_response_by_status_code(&self, status_code: &StatusCode) -> u64 {
        if self.response_count_by_status_and_zoom.contains_key(status_code) {
            self.response_count_by_status_and_zoom[status_code].iter().sum()
        } else {
            0
        }
    }

    fn count_response_by_zoom_level(&self, zoom: u32) -> u64 {
        let mut total = 0;
        for counts_by_zoom in self.response_count_by_status_and_zoom.values() {
            if counts_by_zoom.len() > (zoom as usize) {
                total += counts_by_zoom[zoom as usize];
            }
        }
        return total;
    }

    fn count_response_by_status_code_and_zoom_level(&self, status_code: &StatusCode, zoom: u32) -> u64 {
        if self.response_count_by_status_and_zoom.contains_key(status_code) {
            let counts_by_zoom = &(self.response_count_by_status_and_zoom[status_code]);
            if counts_by_zoom.len() > (zoom as usize) {
                return counts_by_zoom[zoom as usize];
            }
        }
        return 0;
    }

    fn count_total_tile_response(&self) -> u64 {
        self.tile_handle_count_by_source_and_age.iter().sum()
    }

    fn tally_total_tile_response_duration(&self) -> u64 {
        let total_duration = self.tile_handle_duration_by_zoom.iter().fold(
            Duration::zero(),
            |acc, duration| acc + *duration
        );
        return total_duration.num_seconds() as u64
    }

    fn count_tile_response_by_zoom_level(&self, zoom: u32) -> u64 {
        if self.tile_reponse_count_by_zoom.len() > (zoom as usize) {
            self.tile_reponse_count_by_zoom[zoom as usize]
        } else {
            0
        }
    }

    fn tally_tile_response_duration_by_zoom_level(&self, zoom: u32) -> u64 {
        if self.tile_handle_duration_by_zoom.len() > (zoom as usize) {
            self.tile_handle_duration_by_zoom[zoom as usize].num_seconds() as u64
        } else {
            0
        }
    }
}

impl CacheMetrics for ResponseAnalysis {
    fn count_tile_cache_hit_by_age(&self, age: &TileAge) -> u64 {
        *(self.tile_handle_count_by_source_and_age.read(&TileSource::Cache, age))
    }
}

impl RenderMetrics for ResponseAnalysis {
    fn count_tile_renders_by_age(&self, age: &TileAge) -> u64 {
        *(self.tile_handle_count_by_source_and_age.read(&TileSource::Render, age))
    }
}

struct TileMetricTable<T>
where T: Default,
{
    table: [T; TileSource::VARIANT_COUNT * TileAge::VARIANT_COUNT],
}

impl<T: Default> TileMetricTable<T> {
    fn new() -> TileMetricTable<T> {
        TileMetricTable::<T> {
            table: Default::default(),
        }
    }

    fn index(source: &TileSource, age: &TileAge) -> usize {
        (*source as usize * TileAge::VARIANT_COUNT) + *age as usize
    }

    fn read(&self, source: &TileSource, age: &TileAge) -> &T {
        &(self.table[Self::index(source, age)])
    }

    fn iter(&self) -> Iter<'_, T> {
        self.table.iter()
    }

    fn update(&mut self, source: &TileSource, age: &TileAge) -> &mut T {
        &mut (self.table[Self::index(source, age)])
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::apache2::request_rec;
    use crate::schema::apache2::virtual_host::VirtualHost;
    use crate::schema::apache2::config::ModuleConfig;
    use crate::schema::handler::result::HandleOutcome;
    use crate::schema::http::response::HttpResponse;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome };
    use crate::interface::apache2::pool::PoolStored;
    use crate::interface::handler::test_utils::MockRequestHandler;
    use crate::interface::telemetry::metrics::test_utils::with_mock_zero_metrics;
    use crate::apache2::connection::Connection;
    use crate::apache2::request::test_utils::with_request_rec;
    use crate::apache2::request::Apache2Request;
    use crate::apache2::response::Apache2Response;
    use chrono::Utc;
    use http::header::HeaderMap;
    use http::status::StatusCode;
    use std::error::Error;
    use std::ffi::CString;

    fn mock_write(
        _context: &mut WriteContext,
        _response: &response::SlippyResponse
    ) -> WriteResponseResult {
        return Ok(WriteOutcome::NotWritten)
    }

    #[test]
    fn test_tile_handle_duration_accural_on_serve_tile_handle() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let after_timestamp = before_timestamp + Duration::seconds(2);
                let handle_result = HandleRequestResult {
                    before_timestamp,
                    after_timestamp,
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Tile(
                                    response::TileResponse {
                                        source: TileSource::Render,
                                        age: TileAge::Fresh,
                                    }
                                ),
                            }
                        )
                    ),
                };
                let mock_handler = MockRequestHandler { };
                let mut analysis = ResponseAnalysis::new();
                analysis.on_handle(&mock_handler, &handle_context, &read_result, &handle_result);
                assert_eq!(
                    Duration::seconds(2).num_seconds() as u64,
                    analysis.tally_tile_response_duration_by_zoom_level(3),
                    "Handle duration not accrued"
                );
                assert_eq!(
                    Duration::zero().num_seconds() as u64,
                    analysis.tally_tile_response_duration_by_zoom_level(2),
                    "Handle duration does not default to 0"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_tile_handle_duration_accural_on_description_handle() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
                            ),
                            body: request::BodyVariant::DescribeLayer,
                        }
                    )
                );
                let before_timestamp = Utc::now();
                let after_timestamp = before_timestamp + Duration::seconds(3);
                let handle_result = HandleRequestResult {
                    before_timestamp,
                    after_timestamp,
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
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
                    ),
                };
                let mock_handler = MockRequestHandler { };
                let mut analysis = ResponseAnalysis::new();
                analysis.on_handle(&mock_handler, &handle_context, &read_result, &handle_result);
                assert_eq!(
                    0,
                    analysis.tally_total_tile_response_duration(),
                    "Tile handle duration accrued on layer description handle"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_tile_handle_duration_accural_on_invalid_zoom_level() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let handle_result = HandleRequestResult {
                    before_timestamp: Utc::now(),
                    after_timestamp: Utc::now(),
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Tile(
                                    response::TileResponse {
                                        source: TileSource::Render,
                                        age: TileAge::Fresh,
                                    }
                                ),
                            }
                        )
                    ),
                };
                let mock_handler = MockRequestHandler { };
                let mut analysis = ResponseAnalysis::new();
                analysis.on_handle(&mock_handler, &handle_context, &read_result, &handle_result);
                assert_eq!(
                    0,
                    analysis.tally_total_tile_response_duration(),
                    "An accrued duration exists for invalid zoom level"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_count_increment_on_serve_tile_v3_write() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let handle_result = HandleRequestResult {
                    before_timestamp: Utc::now(),
                    after_timestamp: Utc::now(),
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Tile(
                                    response::TileResponse {
                                        source: TileSource::Render,
                                        age: TileAge::Fresh,
                                    }
                                ),
                            }
                        )
                    ),
                };
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
                let write_record = request as *mut request_rec;
                let mut response = Apache2Response::from(unsafe { write_record.as_mut().unwrap() });
                let mut context = WriteContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request).unwrap(),
                    connection: Connection::find_or_make_new(request).unwrap(),
                    response: &mut response,
                };
                analysis.on_write(mock_write, &mut context, &read_result, &handle_result, &write_result);
                assert_eq!(
                    1,
                    analysis.count_response_by_status_code_and_zoom_level(&StatusCode::OK, 3),
                    "Response count not updated"
                );
                assert_eq!(
                    0,
                    analysis.count_response_by_status_code_and_zoom_level(&StatusCode::OK, 2),
                    "Response count does not default to 0"
                );
                assert_eq!(
                    1,
                    analysis.count_tile_response_by_zoom_level(3),
                    "Tile count not updated"
                );
                assert_eq!(
                    0,
                    analysis.count_tile_response_by_zoom_level(2),
                    "Tile count does not default to 0"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_count_increment_on_tile_render() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    connection: Connection::find_or_make_new(request)?,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let after_timestamp = before_timestamp + Duration::seconds(2);
                let handle_result = HandleRequestResult {
                    before_timestamp,
                    after_timestamp,
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Tile(
                                    response::TileResponse {
                                        source: TileSource::Render,
                                        age: TileAge::Fresh,
                                    }
                                ),
                            }
                        )
                    ),
                };
                let mock_handler = MockRequestHandler { };
                let mut analysis = ResponseAnalysis::new();
                analysis.on_handle(&mock_handler, &handle_context, &read_result, &handle_result);
                assert_eq!(
                    0,
                    analysis.count_tile_cache_hit_by_age(&TileAge::Old),
                    "Tile handle count does not default to 0"
                );
                assert_eq!(
                    1,
                    analysis.count_tile_renders_by_age(&TileAge::Fresh),
                    "Tile handle count not incremented"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_count_increment_on_tile_cache() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let after_timestamp = before_timestamp + Duration::seconds(2);
                let handle_result = HandleRequestResult {
                    before_timestamp,
                    after_timestamp,
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Tile(
                                    response::TileResponse {
                                        source: TileSource::Cache,
                                        age: TileAge::VeryOld,
                                    }
                                ),
                            }
                        )
                    ),
                };
                let mock_handler = MockRequestHandler { };
                let mut analysis = ResponseAnalysis::new();
                analysis.on_handle(&mock_handler, &handle_context, &read_result, &handle_result);
                assert_eq!(
                    0,
                    analysis.count_tile_renders_by_age(&TileAge::Old),
                    "Tile handle count does not default to 0"
                );
                assert_eq!(
                    1,
                    analysis.count_tile_cache_hit_by_age(&TileAge::VeryOld),
                    "Tile handle count not incremented"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_count_increment_on_tile_response_combinations() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let mut analysis = ResponseAnalysis::new();
                let all_sources = [TileSource::Render, TileSource::Cache];
                let all_ages = [TileAge::Fresh, TileAge::Old, TileAge::VeryOld];
                for source in &all_sources {
                    for age in &all_ages {
                        let before_timestamp = Utc::now();
                        let after_timestamp = before_timestamp + Duration::seconds(2);
                        let handle_result = HandleRequestResult {
                            before_timestamp,
                            after_timestamp,
                            result: Ok(
                                HandleOutcome::Handled(
                                    response::SlippyResponse {
                                        header: response::Header::new(
                                            handle_context.request.record,
                                            handle_context.connection.record,
                                            handle_context.host.record,
                                            &mime::APPLICATION_JSON,
                                        ),
                                        body: response::BodyVariant::Tile(
                                            response::TileResponse {
                                                source: source.clone(),
                                                age: age.clone(),
                                            }
                                        ),
                                    }
                                )
                            ),
                        };
                        let mock_handler = MockRequestHandler { };
                        analysis.on_handle(&mock_handler, &handle_context, &read_result, &handle_result);
                        analysis.on_handle(&mock_handler, &handle_context, &read_result, &handle_result);
                    }
                }
                for age in &all_ages {
                    assert_eq!(
                        2,
                        analysis.count_tile_cache_hit_by_age(age),
                        "Tile handle count not incremented"
                    );
                    assert_eq!(
                        2,
                        analysis.count_tile_renders_by_age(age),
                        "Tile handle count not incremented"
                    );
                }
                Ok(())
            })
        })
    }

    #[test]
    fn test_count_increment_on_serve_tile_v2_write() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let handle_result = HandleRequestResult {
                    before_timestamp: Utc::now(),
                    after_timestamp: Utc::now(),
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Tile(
                                    response::TileResponse {
                                        source: TileSource::Render,
                                        age: TileAge::Fresh,
                                    }
                                ),
                            }
                        )
                    )
                };
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
                let write_record = request as *mut request_rec;
                let mut response = Apache2Response::from(unsafe { write_record.as_mut().unwrap() });
                let context = WriteContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    response: &mut response,
                };
                analysis.on_write(mock_write, &context, &read_result, &handle_result, &write_result);
                assert_eq!(
                    1,
                    analysis.count_response_by_status_code_and_zoom_level(&StatusCode::OK, 3),
                    "Response count not updated"
                );
                assert_eq!(
                    0,
                    analysis.count_response_by_status_code_and_zoom_level(&StatusCode::OK, 2),
                    "Response count does not default to 0"
                );
                assert_eq!(
                    1,
                    analysis.count_tile_response_by_zoom_level(3),
                    "Tile count not updated"
                );
                assert_eq!(
                    0,
                    analysis.count_tile_response_by_zoom_level(2),
                    "Tile count does not default to 0"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_no_increment_on_description_write() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
                            ),
                            body: request::BodyVariant::DescribeLayer,
                        }
                    )
                );
                let handle_result = HandleRequestResult {
                    before_timestamp: Utc::now(),
                    after_timestamp: Utc::now(),
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
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
                    ),
                };
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
                let write_record = request as *mut request_rec;
                let mut response = Apache2Response::from(unsafe { write_record.as_mut().unwrap() });
                let context = WriteContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    response: &mut response,
                };
                analysis.on_write(mock_write, &context, &read_result, &handle_result, &write_result);
                assert_eq!(
                    0,
                    analysis.count_response_by_status_code(&StatusCode::OK),
                    "Response count incremented on layer description response"
                );
                assert_eq!(
                    0,
                    analysis.count_total_tile_response(),
                    "Tile response count incremented on layer description response"
                );
                Ok(())
            })
        })
    }

    #[test]
    fn test_reponse_count_increment_on_invalid_zoom_level() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let module_config = ModuleConfig::new();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                handle_context.request.record,
                                handle_context.connection.record,
                                handle_context.host.record,
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
                let handle_result = HandleRequestResult {
                    before_timestamp: Utc::now(),
                    after_timestamp: Utc::now(),
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    handle_context.request.record,
                                    handle_context.connection.record,
                                    handle_context.host.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Tile(
                                    response::TileResponse {
                                        source: TileSource::Render,
                                        age: TileAge::Fresh,
                                    }
                                ),
                            }
                        )
                    ),
                };
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
                let write_record = request as *mut request_rec;
                let mut response = Apache2Response::from(unsafe { write_record.as_mut().unwrap() });
                let context = WriteContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_make_new(request)?,
                    response: &mut response,
                };
                analysis.on_write(mock_write, &context, &read_result, &handle_result, &write_result);
                assert_eq!(
                    0,
                    analysis.count_total_tile_response(),
                    "A tile response with an invalid zoom level was counted"
                );
                Ok(())
            })
        })
    }
}
