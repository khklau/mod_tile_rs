use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::error::WriteError;
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::identity::LayerName;
use crate::schema::tile::source::TileSource;
use crate::io::communication::interface::HttpResponseWriter;
use crate::adapter::slippy::interface::{WriteContext, WriteResponseObserver,};
use crate::service::telemetry::interface::TileHandlingMetrics;

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
        response: &response::SlippyResponse,
        _writer: &dyn HttpResponseWriter,
        _write_result: &Result<HttpResponse, WriteError>,
        _write_func_name: &'static str,
        request: &request::SlippyRequest,
    ) -> () {
        let handle_duration = response.header.after_timestamp - response.header.before_timestamp;
        match &response.body {
            response::BodyVariant::Tile(response) => self.on_handled_tile(
                context,
                request,
                response,
                &handle_duration,
            ),
            _ => (),
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
            _write_result: &Result<HttpResponse, WriteError>,
            _write_func_name: &'static str,
            _request: &request::SlippyRequest,
        ) -> () {
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::identifier::generate_id;
    use crate::schema::apache2::config::ModuleConfig;
    use crate::schema::http::encoding::ContentEncoding;
    use crate::schema::http::response::HttpResponse;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::schema::tile::tile_ref::TileRef;
    use crate::framework::apache2::context::HostContext;
    use crate::framework::apache2::record::test_utils::with_request_rec;
    use crate::io::communication::http_exchange::test_utils::MockWriter;
    use chrono::Utc;
    use http::header::HeaderMap;
    use http::status::StatusCode;
    use std::cell::RefCell;
    use std::error::Error as StdError;
    use std::ffi::CString;

    #[test]
    fn test_count_increment_on_tile_render() -> Result<(), Box<dyn StdError>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.clone().into_raw();
            let module_config = ModuleConfig::new();
            let slippy_request = request::SlippyRequest {
                header: request::Header {
                    layer: LayerName::new(),
                    request_id: generate_id(),
                    uri: uri.into_string()?,
                    received_timestamp: Utc::now(),
                },
                body: request::BodyVariant::ServeTile(
                    request::ServeTileRequest::V3(
                        request::ServeTileRequestV3 {
                            parameter: String::from("foo"),
                            x: 1,
                            y: 2,
                            z: 3,
                            extension: String::from("jpg"),
                            option: None,
                        }
                    )
                ),
            };
            let context = WriteContext {
                host_context: HostContext::new(&module_config, request),
                request: &slippy_request,
            };
            let before_timestamp = Utc::now();
            let after_timestamp = before_timestamp + Duration::seconds(2);
            let empty_tile: RefCell<Vec<u8>> = RefCell::new(Vec::new());
            let tile_ref = TileRef {
                raw_bytes: empty_tile.clone(),
                begin: 0,
                end: 1,
                media_type: mime::IMAGE_PNG,
                encoding: ContentEncoding::NotCompressed,
            };
            let response = response::SlippyResponse {
                header: response::Header {
                    mime_type: mime::APPLICATION_JSON.clone(),
                    before_timestamp,
                    after_timestamp,
                },
                body: response::BodyVariant::Tile(
                    response::TileResponse {
                        source: TileSource::Render,
                        age: TileAge::Fresh,
                        tile_ref,
                    }
                ),
            };
            let write_result = Ok(
                HttpResponse {
                    status_code: StatusCode::OK,
                    bytes_written: 508,
                    http_headers: HeaderMap::new(),
                }
            );
            let mut analysis = TileHandlingAnalysis::new(&module_config)?;
            let writer = MockWriter::new();
            analysis.on_write(&context, &response, &writer, &write_result, "mock", &slippy_request);
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
    fn test_count_increment_on_tile_cache() -> Result<(), Box<dyn StdError>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.clone().into_raw();
            let module_config = ModuleConfig::new();
            let slippy_request = request::SlippyRequest {
                header: request::Header {
                    layer: LayerName::new(),
                    request_id: generate_id(),
                    uri: uri.into_string()?,
                    received_timestamp: Utc::now(),
                },
                body: request::BodyVariant::ServeTile(
                    request::ServeTileRequest::V3(
                        request::ServeTileRequestV3 {
                            parameter: String::from("foo"),
                            x: 1,
                            y: 2,
                            z: 3,
                            extension: String::from("jpg"),
                            option: None,
                        }
                    )
                ),
            };
            let context = WriteContext {
                host_context: HostContext::new(&module_config, request),
                request: &slippy_request,
            };
            let before_timestamp = Utc::now();
            let after_timestamp = before_timestamp + Duration::seconds(2);
            let empty_tile: RefCell<Vec<u8>> = RefCell::new(Vec::new());
            let tile_ref = TileRef {
                raw_bytes: empty_tile.clone(),
                begin: 0,
                end: 1,
                media_type: mime::IMAGE_PNG,
                encoding: ContentEncoding::NotCompressed,
            };
            let response = response::SlippyResponse {
                header: response::Header {
                    mime_type: mime::APPLICATION_JSON.clone(),
                    before_timestamp,
                    after_timestamp,
                },
                body: response::BodyVariant::Tile(
                    response::TileResponse {
                        source: TileSource::Cache,
                        age: TileAge::VeryOld,
                        tile_ref,
                    }
                ),
            };
            let write_result = Ok(
                HttpResponse {
                    status_code: StatusCode::OK,
                    bytes_written: 508,
                    http_headers: HeaderMap::new(),
                }
            );
            let mut analysis = TileHandlingAnalysis::new(&module_config)?;
            let writer = MockWriter::new();
            analysis.on_write(&context, &response, &writer, &write_result, "mock", &slippy_request);
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
    fn test_count_increment_on_tile_response_combinations() -> Result<(), Box<dyn StdError>> {
        with_request_rec(|request| {
            let uri = CString::new("/mod_tile_rs")?;
            request.uri = uri.clone().into_raw();
            let module_config = ModuleConfig::new();
            let slippy_request = request::SlippyRequest {
                header: request::Header {
                    layer: LayerName::new(),
                    request_id: generate_id(),
                    uri: uri.into_string()?,
                    received_timestamp: Utc::now(),
                },
                body: request::BodyVariant::ServeTile(
                    request::ServeTileRequest::V3(
                        request::ServeTileRequestV3 {
                            parameter: String::from("foo"),
                            x: 1,
                            y: 2,
                            z: 3,
                            extension: String::from("jpg"),
                            option: None,
                        }
                    )
                ),
            };
            let context = WriteContext {
                host_context: HostContext::new(&module_config, request),
                request: &slippy_request,
            };
            let write_result = Ok(
                HttpResponse {
                    status_code: StatusCode::OK,
                    bytes_written: 508,
                    http_headers: HeaderMap::new(),
                }
            );
            let mut analysis = TileHandlingAnalysis::new(&module_config)?;
            let all_sources = [TileSource::Render, TileSource::Cache];
            let all_ages = [TileAge::Fresh, TileAge::Old, TileAge::VeryOld];
            let empty_tile: RefCell<Vec<u8>> = RefCell::new(Vec::new());
            for source in &all_sources {
                for age in &all_ages {
                    let before_timestamp = Utc::now();
                    let after_timestamp = before_timestamp + Duration::seconds(2);
                    let tile_ref = TileRef {
                        raw_bytes: empty_tile.clone(),
                        begin: 0,
                        end: 1,
                        media_type: mime::IMAGE_PNG,
                        encoding: ContentEncoding::NotCompressed,
                    };
                    let response = response::SlippyResponse {
                        header: response::Header {
                            mime_type: mime::APPLICATION_JSON.clone(),
                            before_timestamp,
                            after_timestamp,
                        },
                        body: response::BodyVariant::Tile(
                            response::TileResponse {
                                source: source.clone(),
                                age: age.clone(),
                                tile_ref,
                            }
                        ),
                    };
                    let writer = MockWriter::new();
                    analysis.on_write(&context, &response, &writer, &write_result, "mock", &slippy_request);
                    analysis.on_write(&context, &response, &writer, &write_result, "mock", &slippy_request);
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
