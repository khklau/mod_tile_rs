use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::slippy::result::{ReadOutcome, WriteOutcome,};
use crate::schema::tile::age::TileAge;
use crate::schema::tile::identity::LayerName;
use crate::schema::tile::source::TileSource;
use crate::interface::communication::HttpResponseWriter;
use crate::interface::slippy::{WriteContext, WriteResponseObserver,};
use crate::interface::telemetry::TileHandlingMetrics;

use chrono::Duration;
use enum_iterator::IntoEnumIterator;

use std::collections::hash_map::HashMap;
use std::marker::Copy;


pub struct TileHandlingAnalysis {
    analysis_by_layer: HashMap<LayerName, TileLayerHandlingAnalysis>,
}

impl TileHandlingAnalysis {
    pub fn new(_config: &ModuleConfig) -> Result<TileHandlingAnalysis, InvalidConfigError> {
        Ok(
            TileHandlingAnalysis {
                analysis_by_layer: HashMap::new(),
            }
        )
    }

    fn mut_layer<'s>(
        &'s mut self,
        request: &request::SlippyRequest,
    ) -> &'s mut TileLayerHandlingAnalysis {
        let layer = &request.header.layer;
        self.analysis_by_layer.entry(layer.clone()).or_insert(TileLayerHandlingAnalysis::new())
    }

    fn on_handled_tile(
        &mut self,
        context: &WriteContext,
        request: &request::SlippyRequest,
        response: &response::TileResponse,
        handle_duration: &Duration,
    ) -> () {
        self.increase_tile_handle_count(context, request, response);
        self.accrue_tile_handle_duration(context, request, response, handle_duration);
    }

    fn increase_tile_handle_count(
        &mut self,
        _context: &WriteContext,
        request: &request::SlippyRequest,
        response: &response::TileResponse,
    ) -> () {
        let counter = self.mut_layer(request).tile_handle_count_by_source_and_age.update(
            &response.source,
            &response.age
        );
        *counter += 1;
    }

    fn accrue_tile_handle_duration(
        &mut self,
        _context: &WriteContext,
        request: &request::SlippyRequest,
        response: &response::TileResponse,
        handle_duration: &Duration,
    ) -> () {
        let tally = self.mut_layer(request).tile_handle_duration_by_source_and_age.update(
            &response.source,
            &response.age
        );
        *tally = *tally + *handle_duration;
    }
}

impl WriteResponseObserver for TileHandlingAnalysis {
    fn on_write(
        &mut self,
        context: &WriteContext,
        _response: &response::SlippyResponse,
        _writer: &dyn HttpResponseWriter,
        _write_outcome: &WriteOutcome,
        _write_func_name: &'static str,
        read_outcome: &ReadOutcome,
        handle_outcome: &HandleOutcome,
    ) -> () {
        match (&read_outcome, &handle_outcome) {
            (ReadOutcome::Processed(read_result), HandleOutcome::Processed(handle_result)) => {
                let handle_duration = handle_result.after_timestamp - handle_result.before_timestamp;
                match (read_result, &handle_result.result) {
                    (Ok(request), Ok(response)) => match &response.body {
                        response::BodyVariant::Tile(response) => self.on_handled_tile(
                            context,
                            request,
                            response,
                            &handle_duration,
                        ),
                        _ => (),
                    },
                    _ => (),
                }
            },
            _ => ()
        }
    }
}

struct TileLayerHandlingAnalysis {
    tile_handle_count_by_source_and_age: TileMetricTable<u64>,
    tile_handle_duration_by_source_and_age: TileMetricTable<Duration>,
}

impl TileLayerHandlingAnalysis {
    fn new() -> TileLayerHandlingAnalysis {
        TileLayerHandlingAnalysis {
            tile_handle_count_by_source_and_age: TileMetricTable::new(),
            tile_handle_duration_by_source_and_age: TileMetricTable::new()
        }
    }
}

impl TileHandlingMetrics for TileHandlingAnalysis {
    fn iterate_valid_cache_ages(&self) -> Vec<TileAge> {
        TileAge::into_enum_iter().collect()
    }

    fn iterate_valid_render_ages(&self) -> Vec<TileAge> {
        TileAge::into_enum_iter().collect()
    }

    fn count_handled_tile_by_source_and_age(
        &self,
        source: &TileSource,
        age: &TileAge,
    ) -> u64 {
        let mut total = 0;
        for layer_analysis in self.analysis_by_layer.values() {
            total += *(layer_analysis.tile_handle_count_by_source_and_age.read(source, age));
        }
        return total;
    }

    fn tally_tile_handle_duration_by_source_and_age(
        &self,
        source: &TileSource,
        age: &TileAge,
    ) -> u64 {
        let mut total = Duration::zero();
        for layer_analysis in self.analysis_by_layer.values() {
            total = total + *(layer_analysis.tile_handle_duration_by_source_and_age.read(source, age));
        }
        return total.num_seconds() as u64;
    }
}

trait DefaultMetric {
    fn default() -> Self;
}

impl DefaultMetric for Duration {
    fn default() -> Self {
        Duration::zero()
    }
}

impl DefaultMetric for u64 {
    fn default() -> Self {
        0
    }
}

struct TileMetricTable<T>
where T: DefaultMetric,
{
    table: [T; TileSource::VARIANT_COUNT * TileAge::VARIANT_COUNT],
}

impl<T: DefaultMetric + Copy> TileMetricTable<T> {
    fn new() -> TileMetricTable<T> {
        TileMetricTable::<T> {
            table: [DefaultMetric::default(); TileSource::VARIANT_COUNT * TileAge::VARIANT_COUNT],
        }
    }

    fn index(source: &TileSource, age: &TileAge) -> usize {
        (*source as usize * TileAge::VARIANT_COUNT) + *age as usize
    }

    fn read(&self, source: &TileSource, age: &TileAge) -> &T {
        &(self.table[Self::index(source, age)])
    }

    fn update(&mut self, source: &TileSource, age: &TileAge) -> &mut T {
        &mut (self.table[Self::index(source, age)])
    }
}


#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub struct MockNoOpTileHandlingAnalysis {}

    impl TileHandlingMetrics for MockNoOpTileHandlingAnalysis {
        fn iterate_valid_cache_ages(&self) -> Vec<TileAge> {
            TileAge::into_enum_iter().collect()
        }

        fn iterate_valid_render_ages(&self) -> Vec<TileAge> {
            TileAge::into_enum_iter().collect()
        }

        fn count_handled_tile_by_source_and_age(
            &self,
            _source: &TileSource,
            _age: &TileAge,
        ) -> u64 {
            0
        }

        fn tally_tile_handle_duration_by_source_and_age(
            &self,
            _source: &TileSource,
            _age: &TileAge,
        ) -> u64 {
            0
        }
    }

    impl WriteResponseObserver for MockNoOpTileHandlingAnalysis {
        fn on_write(
            &mut self,
            _context: &WriteContext,
            _response: &response::SlippyResponse,
            _writer: &dyn HttpResponseWriter,
            _write_outcome: &WriteOutcome,
            _write_func_name: &'static str,
            _read_outcome: &ReadOutcome,
            _handle_outcome: &HandleOutcome,
        ) -> () {
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::apache2::config::ModuleConfig;
    use crate::schema::apache2::connection::Connection;
    use crate::schema::apache2::request::Apache2Request;
    use crate::schema::apache2::virtual_host::VirtualHost;
    use crate::schema::handler::result::HandleRequestResult;
    use crate::schema::http::encoding::ContentEncoding;
    use crate::schema::http::response::HttpResponse;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::schema::slippy::result::WriteOutcome;
    use crate::interface::apache2::PoolStored;
    use crate::interface::tile::TileRef;
    use crate::framework::apache2::record::test_utils::with_request_rec;
    use crate::framework::apache2::writer::test_utils::MockWriter;
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
            let write_context = WriteContext {
                module_config: &module_config,
                connection: Connection::find_or_allocate_new(request)?,
                host: VirtualHost::find_or_allocate_new(request)?,
                request: Apache2Request::create_with_tile_config(request)?,
            };
            let read_outcome = ReadOutcome::Processed(
                Ok(
                    request::SlippyRequest {
                        header: request::Header::new(
                            write_context.request.record,
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
                    write_context.request.record,
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
            let mut analysis = TileHandlingAnalysis::new(&module_config)?;
            let writer = MockWriter::new();
            analysis.on_write(&write_context, &response, &writer, &write_outcome, "mock", &read_outcome, &handle_outcome);
            assert_eq!(
                0,
                analysis.count_handled_tile_by_source_and_age(&TileSource::Cache, &TileAge::Old),
                "Tile handle count does not default to 0"
            );
            assert_eq!(
                1,
                analysis.count_handled_tile_by_source_and_age(&TileSource::Render, &TileAge::Fresh),
                "Tile handle count not incremented"
            );
            Ok(())
        })
    }

    #[test]
    fn test_count_increment_on_tile_cache() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let module_config = ModuleConfig::new();
            let write_context = WriteContext {
                module_config: &module_config,
                host: VirtualHost::find_or_allocate_new(request)?,
                connection: Connection::find_or_allocate_new(request)?,
                request: Apache2Request::create_with_tile_config(request)?,
            };
            let read_outcome = ReadOutcome::Processed(
                Ok(
                    request::SlippyRequest {
                        header: request::Header::new(
                            write_context.request.record,
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
                    write_context.request.record,
                    &mime::APPLICATION_JSON,
                ),
                body: response::BodyVariant::Tile(
                    response::TileResponse {
                        source: TileSource::Cache,
                        age: TileAge::VeryOld,
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
            let mut analysis = TileHandlingAnalysis::new(&module_config)?;
            let writer = MockWriter::new();
            analysis.on_write(&write_context, &response, &writer, &write_outcome, "mock", &read_outcome, &handle_outcome);
            assert_eq!(
                0,
                analysis.count_handled_tile_by_source_and_age(&TileSource::Render, &TileAge::Old),
                "Tile handle count does not default to 0"
            );
            assert_eq!(
                1,
                analysis.count_handled_tile_by_source_and_age(&TileSource::Cache, &TileAge::VeryOld),
                "Tile handle count not incremented"
            );
            Ok(())
        })
    }

    #[test]
    fn test_count_increment_on_tile_response_combinations() -> Result<(), Box<dyn Error>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.into_raw();
            let module_config = ModuleConfig::new();
            let write_context = WriteContext {
                module_config: &module_config,
                host: VirtualHost::find_or_allocate_new(request)?,
                connection: Connection::find_or_allocate_new(request)?,
                request: Apache2Request::create_with_tile_config(request)?,
            };
            let read_outcome = ReadOutcome::Processed(
                Ok(
                    request::SlippyRequest {
                        header: request::Header::new(
                            write_context.request.record,
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
            let write_outcome = WriteOutcome::Processed(
                Ok(
                    HttpResponse {
                        status_code: StatusCode::OK,
                        bytes_written: 508,
                        http_headers: HeaderMap::new(),
                    }
                )
            );
            let mut analysis = TileHandlingAnalysis::new(&module_config)?;
            let all_sources = [TileSource::Render, TileSource::Cache];
            let all_ages = [TileAge::Fresh, TileAge::Old, TileAge::VeryOld];
            let empty_tile: Rc<Vec<u8>> = Rc::new(Vec::new());
            for source in &all_sources {
                for age in &all_ages {
                    let before_timestamp = Utc::now();
                    let after_timestamp = before_timestamp + Duration::seconds(2);
                    let tile_ref = TileRef {
                        raw_bytes: Rc::downgrade(&empty_tile),
                        begin: 0,
                        end: 1,
                        media_type: mime::IMAGE_PNG,
                        encoding: ContentEncoding::NotCompressed,
                    };
                    let response = response::SlippyResponse {
                        header: response::Header::new(
                            write_context.request.record,
                            &mime::APPLICATION_JSON,
                        ),
                        body: response::BodyVariant::Tile(
                            response::TileResponse {
                                source: source.clone(),
                                age: age.clone(),
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
                    let writer = MockWriter::new();
                    analysis.on_write(&write_context, &response, &writer, &write_outcome, "mock", &read_outcome, &handle_outcome);
                    analysis.on_write(&write_context, &response, &writer, &write_outcome, "mock", &read_outcome, &handle_outcome);
                }
            }
            for age in &all_ages {
                assert_eq!(
                    2,
                    analysis.count_handled_tile_by_source_and_age(&TileSource::Cache, age),
                    "Tile handle count not incremented"
                );
                assert_eq!(
                    2,
                    analysis.count_handled_tile_by_source_and_age(&TileSource::Render, age),
                    "Tile handle count not incremented"
                );
            }
            Ok(())
        })
    }
}
