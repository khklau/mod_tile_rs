use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::interface::handler::{
    HandleContext, HandleContext2, HandleIOContext, RequestHandler, RequestHandler2,
};
use crate::interface::telemetry::TelemetryInventory;

use chrono::Utc;
use http::status::StatusCode;
use mime;

use std::any::type_name;


pub struct StatisticsHandlerState { }

impl StatisticsHandlerState {
    pub fn new(_config: &ModuleConfig) -> Result<StatisticsHandlerState, InvalidConfigError> {
        Ok(
            StatisticsHandlerState {  }
        )
    }

    fn report(
        &self,
        context: &HandleContext2
    ) -> response::Statistics {
        let mut result = response::Statistics::new();
        for status_code in context.telemetry.response_metrics().iterate_status_codes_responded() {
            let count = context.telemetry.response_metrics().count_response_by_status_code(status_code);
            match status_code {
                &StatusCode::OK => { result.number_response_200 = count; },
                &StatusCode::NOT_MODIFIED => { result.number_response_304 = count; },
                &StatusCode::NOT_FOUND => { result.number_response_404 = count; },
                &StatusCode::SERVICE_UNAVAILABLE => { result.number_response_503 = count; },
                &StatusCode::INTERNAL_SERVER_ERROR => { result.number_response_5xx = count; },
                _ => { result.number_response_other += count; }
            }
        }
        result.number_fresh_cache = context.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Fresh,
        );
        result.number_old_cache = context.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Old,
        );
        result.number_very_old_cache = context.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::VeryOld,
        );
        result.number_fresh_render = context.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Fresh,
        );
        result.number_old_render = context.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Old,
        );
        result.number_very_old_render = context.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::VeryOld,
        );
        for zoom_level in context.telemetry.response_metrics().iterate_valid_zoom_levels() {
            let any_count = context.telemetry.response_metrics().count_response_by_zoom_level(zoom_level);
            result.number_successful_response_by_zoom[zoom_level as usize] = any_count;
            let tile_count = context.telemetry.response_metrics().count_tile_response_by_zoom_level(zoom_level);
            result.number_tile_response_by_zoom[zoom_level as usize] = tile_count;
            let tile_duration = context.telemetry.response_metrics().tally_tile_response_duration_by_zoom_level(zoom_level);
            result.duration_tile_response_by_zoom[zoom_level as usize] = tile_duration;
        }
        result.total_number_tile_response = context.telemetry.response_metrics().count_total_tile_response();
        result.total_duration_tile_response = context.telemetry.response_metrics().tally_total_tile_response_duration();
        for layer in context.telemetry.response_metrics().iterate_layers_responded() {
            let count_200 = context.telemetry.response_metrics().count_response_by_layer_and_status_code(layer, &http::StatusCode::OK);
            result.number_response_200_by_layer.insert(String::from(layer.as_str()), count_200);
            let count_404 = context.telemetry.response_metrics().count_response_by_layer_and_status_code(layer, &http::StatusCode::NOT_FOUND);
            result.number_response_404_by_layer.insert(String::from(layer.as_str()), count_404);
        }
        return result;
    }
}

impl RequestHandler2 for StatisticsHandlerState {
    fn handle2(
        &mut self,
        context: &HandleContext2,
        _io: &mut HandleIOContext,
        request: &request::SlippyRequest,
    ) -> HandleOutcome {
        let before_timestamp = Utc::now();
        match request.body {
            request::BodyVariant::ReportStatistics => (),
            _ => {
                return HandleOutcome::Ignored;
            },
        };
        let statistics = self.report(context);
        let response = response::SlippyResponse {
            header: response::Header::new(
                context.request.record,
                &mime::TEXT_PLAIN,
            ),
            body: response::BodyVariant::Statistics(statistics),
        };
        let after_timestamp = Utc::now();
        return HandleOutcome::Processed(
            HandleRequestResult {
                before_timestamp,
                after_timestamp,
                result: Ok(response),
            }
        );
    }

    fn type_name2(&self) -> &'static str {
        type_name::<Self>()
    }
}

pub struct StatisticsHandler<'h> {
    _state: &'h mut StatisticsHandlerState,
    telemetry: &'h dyn TelemetryInventory,
}

impl<'h> StatisticsHandler<'h> {
    pub fn new(
        state: &'h mut StatisticsHandlerState,
        telemetry: &'h dyn TelemetryInventory
    ) -> StatisticsHandler<'h> {
        StatisticsHandler {
            _state: state,
            telemetry,
        }
    }

    fn report(
        &self,
        _context: &HandleContext
    ) -> response::Statistics {
        let mut result = response::Statistics::new();
        for status_code in self.telemetry.response_metrics().iterate_status_codes_responded() {
            let count = self.telemetry.response_metrics().count_response_by_status_code(status_code);
            match status_code {
                &StatusCode::OK => { result.number_response_200 = count; },
                &StatusCode::NOT_MODIFIED => { result.number_response_304 = count; },
                &StatusCode::NOT_FOUND => { result.number_response_404 = count; },
                &StatusCode::SERVICE_UNAVAILABLE => { result.number_response_503 = count; },
                &StatusCode::INTERNAL_SERVER_ERROR => { result.number_response_5xx = count; },
                _ => { result.number_response_other += count; }
            }
        }
        result.number_fresh_cache = self.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Fresh,
        );
        result.number_old_cache = self.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Old,
        );
        result.number_very_old_cache = self.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::VeryOld,
        );
        result.number_fresh_render = self.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Fresh,
        );
        result.number_old_render = self.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Old,
        );
        result.number_very_old_render = self.telemetry.tile_handling_metrics().count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::VeryOld,
        );
        for zoom_level in self.telemetry.response_metrics().iterate_valid_zoom_levels() {
            let any_count = self.telemetry.response_metrics().count_response_by_zoom_level(zoom_level);
            result.number_successful_response_by_zoom[zoom_level as usize] = any_count;
            let tile_count = self.telemetry.response_metrics().count_tile_response_by_zoom_level(zoom_level);
            result.number_tile_response_by_zoom[zoom_level as usize] = tile_count;
            let tile_duration = self.telemetry.response_metrics().tally_tile_response_duration_by_zoom_level(zoom_level);
            result.duration_tile_response_by_zoom[zoom_level as usize] = tile_duration;
        }
        result.total_number_tile_response = self.telemetry.response_metrics().count_total_tile_response();
        result.total_duration_tile_response = self.telemetry.response_metrics().tally_total_tile_response_duration();
        for layer in self.telemetry.response_metrics().iterate_layers_responded() {
            let count_200 = self.telemetry.response_metrics().count_response_by_layer_and_status_code(layer, &http::StatusCode::OK);
            result.number_response_200_by_layer.insert(String::from(layer.as_str()), count_200);
            let count_404 = self.telemetry.response_metrics().count_response_by_layer_and_status_code(layer, &http::StatusCode::NOT_FOUND);
            result.number_response_404_by_layer.insert(String::from(layer.as_str()), count_404);
        }
        return result;
    }
}

impl<'h> RequestHandler for StatisticsHandler<'h> {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &request::SlippyRequest,
    ) -> HandleOutcome {
        let before_timestamp = Utc::now();
        match request.body {
            request::BodyVariant::ReportStatistics => (),
            _ => {
                return HandleOutcome::Ignored;
            },
        };
        let statistics = self.report(context);
        let response = response::SlippyResponse {
            header: response::Header::new(
                context.request.record,
                &mime::TEXT_PLAIN,
            ),
            body: response::BodyVariant::Statistics(statistics),
        };
        let after_timestamp = Utc::now();
        return HandleOutcome::Processed(
            HandleRequestResult {
                before_timestamp,
                after_timestamp,
                result: Ok(response),
            }
        );
    }

    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::apache2::config::ModuleConfig;
    use crate::schema::apache2::connection::Connection;
    use crate::schema::apache2::request::Apache2Request;
    use crate::schema::apache2::virtual_host::VirtualHost;
    use crate::schema::tile::identity::LayerName;
    use crate::interface::apache2::PoolStored;
    use crate::interface::handler::HandleRequestObserver;
    use crate::interface::communication::test_utils::EmptyResultCommunicationInventory;
    use crate::interface::slippy::{ReadRequestObserver, WriteResponseObserver,};
    use crate::interface::storage::test_utils::BlankStorageInventory;
    use crate::interface::telemetry::{ResponseMetrics, TileHandlingMetrics, TelemetryInventory,};
    use crate::interface::telemetry::test_utils::{with_mock_zero_metrics, NoOpZeroTelemetryInventory,};
    use crate::framework::apache2::record::test_utils::with_request_rec;

    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn test_not_handled() -> Result<(), Box<dyn Error>> {
        with_mock_zero_metrics(|response_metrics, tile_handling_metrics| {
            let telemetry_inventory = NoOpZeroTelemetryInventory::new();
            let module_config = ModuleConfig::new();
            let mut stat_state = StatisticsHandlerState::new(&module_config)?;
            let mut stat_handler = StatisticsHandler::new(&mut stat_state, &telemetry_inventory);
            let layer_name = LayerName::from("default");
            let layer_config = module_config.layers.get(&layer_name).unwrap();
            with_request_rec(|request| {
                let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
                request.uri = uri.into_raw();
                let handle_context = HandleContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(request)?,
                    connection: Connection::find_or_allocate_new(request)?,
                    request: Apache2Request::create_with_tile_config(request)?,
                };
                let request = request::SlippyRequest {
                    header: request::Header::new_with_layer(
                        handle_context.request.record,
                        &layer_name,
                    ),
                    body: request::BodyVariant::DescribeLayer,
                };

                assert!(stat_handler.handle(&handle_context, &request).is_ignored(), "Expected to not handle");
                Ok(())
            })
        })
    }

    #[test]
    fn test_not_handled2() -> Result<(), Box<dyn Error>> {
        with_mock_zero_metrics(|response_metrics, tile_handling_metrics| {
            let telemetry = NoOpZeroTelemetryInventory::new();
            let mut communication = EmptyResultCommunicationInventory::new();
            let mut storage = BlankStorageInventory::new();
            let module_config = ModuleConfig::new();
            let mut stat_state = StatisticsHandlerState::new(&module_config)?;
            let layer_name = LayerName::from("default");
            let layer_config = module_config.layers.get(&layer_name).unwrap();
            with_request_rec(|request| {
                let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
                request.uri = uri.into_raw();
                let handle_context = HandleContext2::new(
                    request,
                    &module_config,
                    &telemetry,
                );
                let mut io_context = HandleIOContext::new(
                    &mut communication,
                    &mut storage,
                );
                let request = request::SlippyRequest {
                    header: request::Header::new_with_layer(
                        handle_context.request.record,
                        &layer_name,
                    ),
                    body: request::BodyVariant::DescribeLayer,
                };

                assert!(stat_state.handle2(&handle_context, &mut io_context, &request).is_ignored(), "Expected to not handle");
                Ok(())
            })
        })
    }
}
