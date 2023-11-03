use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::interface::context::IOContext;
use crate::interface::handler::{HandleContext, RequestHandler,};

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
        context: &HandleContext
    ) -> response::Statistics {
        let mut result = response::Statistics::new();
        let response_metrics = context.telemetry.response_metrics();
        let tile_handling_metrics = context.telemetry.tile_handling_metrics();
        for status_code in response_metrics.iterate_status_codes_responded() {
            let count = response_metrics.count_response_by_status_code(&status_code);
            match &status_code {
                &StatusCode::OK => { result.number_response_200 = count; },
                &StatusCode::NOT_MODIFIED => { result.number_response_304 = count; },
                &StatusCode::NOT_FOUND => { result.number_response_404 = count; },
                &StatusCode::SERVICE_UNAVAILABLE => { result.number_response_503 = count; },
                &StatusCode::INTERNAL_SERVER_ERROR => { result.number_response_5xx = count; },
                _ => { result.number_response_other += count; }
            }
        }
        result.number_fresh_cache = tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Fresh,
        );
        result.number_old_cache = tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Old,
        );
        result.number_very_old_cache = tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::VeryOld,
        );
        result.number_fresh_render = tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Fresh,
        );
        result.number_old_render = tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Old,
        );
        result.number_very_old_render = tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::VeryOld,
        );
        for zoom_level in response_metrics.iterate_valid_zoom_levels() {
            let any_count = response_metrics.count_response_by_zoom_level(zoom_level);
            result.number_successful_response_by_zoom[zoom_level as usize] = any_count;
            let tile_count = response_metrics.count_tile_response_by_zoom_level(zoom_level);
            result.number_tile_response_by_zoom[zoom_level as usize] = tile_count;
            let tile_duration = response_metrics.tally_tile_response_duration_by_zoom_level(zoom_level);
            result.duration_tile_response_by_zoom[zoom_level as usize] = tile_duration;
        }
        result.total_number_tile_response = response_metrics.count_total_tile_response();
        result.total_duration_tile_response = response_metrics.tally_total_tile_response_duration();
        for layer in response_metrics.iterate_layers_responded() {
            let count_200 = response_metrics.count_response_by_layer_and_status_code(&layer, &http::StatusCode::OK);
            result.number_response_200_by_layer.insert(String::from(layer.as_str()), count_200);
            let count_404 = response_metrics.count_response_by_layer_and_status_code(&layer, &http::StatusCode::NOT_FOUND);
            result.number_response_404_by_layer.insert(String::from(layer.as_str()), count_404);
        }
        return result;
    }
}

impl RequestHandler for StatisticsHandlerState {
    fn handle(
        &mut self,
        context: &HandleContext,
        _io: &mut IOContext,
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
    use crate::interface::communication::test_utils::EmptyResultCommunicationInventory;
    use crate::interface::handler::HandleRequestObserver;
    use crate::interface::handler::test_utils::NoOpHandleRequestObserver;
    use crate::interface::slippy::{ReadRequestObserver, WriteResponseObserver,};
    use crate::interface::slippy::test_utils::{NoOpReadRequestObserver, NoOpWriteResponseObserver,};
    use crate::interface::storage::test_utils::BlankStorageInventory;
    use crate::interface::telemetry::{
        MockResponseMetrics, MockTileHandlingMetrics,
        ResponseMetrics, TileHandlingMetrics, TelemetryInventory
    };
    use crate::interface::telemetry::test_utils::NoOpZeroTelemetryInventory;
    use crate::framework::apache2::record::test_utils::with_request_rec;

    use mockall::predicate::eq;

    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn test_not_handled() -> Result<(), Box<dyn Error>> {
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
            let handle_context = HandleContext::new(
                request,
                &module_config,
                &telemetry,
            );
            let mut io_context = IOContext::new(
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

            assert!(stat_state.handle(&handle_context, &mut io_context, &request).is_ignored(), "Expected to not handle");
            Ok(())
        })
    }

    pub struct TelemetryInventoryWithMockedMetrics {
        response_metrics: MockResponseMetrics,
        tile_handling_metrics: MockTileHandlingMetrics,
        read_observer_0: NoOpReadRequestObserver,
        read_observer_1: NoOpReadRequestObserver,
        handle_observer_0: NoOpHandleRequestObserver,
        handle_observer_1: NoOpHandleRequestObserver,
        write_observer_0: NoOpWriteResponseObserver,
        write_observer_1: NoOpWriteResponseObserver,
        write_observer_2: NoOpWriteResponseObserver,
        write_observer_3: NoOpWriteResponseObserver,
    }

    impl TelemetryInventoryWithMockedMetrics {
        pub fn new() -> TelemetryInventoryWithMockedMetrics {
            TelemetryInventoryWithMockedMetrics {
                response_metrics: MockResponseMetrics::new(),
                tile_handling_metrics: MockTileHandlingMetrics::new(),
                read_observer_0: NoOpReadRequestObserver::new(),
                read_observer_1: NoOpReadRequestObserver::new(),
                handle_observer_0: NoOpHandleRequestObserver::new(),
                handle_observer_1: NoOpHandleRequestObserver::new(),
                write_observer_0: NoOpWriteResponseObserver::new(),
                write_observer_1: NoOpWriteResponseObserver::new(),
                write_observer_2: NoOpWriteResponseObserver::new(),
                write_observer_3: NoOpWriteResponseObserver::new(),
            }
        }
    }

    impl TelemetryInventory for TelemetryInventoryWithMockedMetrics {
        fn response_metrics(&self) -> &dyn ResponseMetrics { &self.response_metrics }

        fn tile_handling_metrics(&self) -> &dyn TileHandlingMetrics { &self.tile_handling_metrics }

        fn read_request_observers(&mut self) -> [&mut dyn ReadRequestObserver; 2] {
            [&mut self.read_observer_0, &mut self.read_observer_1]
        }

        fn handle_request_observers(&mut self) -> [&mut dyn HandleRequestObserver; 2] {
            [&mut self.handle_observer_0, &mut self.handle_observer_1]
        }

        fn write_response_observers(&mut self) -> [&mut dyn WriteResponseObserver; 4] {
            [
                &mut self.write_observer_0,
                &mut self.write_observer_1,
                &mut self.write_observer_2,
                &mut self.write_observer_3,
            ]
        }
    }

    #[test]
    fn test_handle_after_tile_render() -> Result<(), Box<dyn Error>> {
        let module_config = ModuleConfig::new();
        let mut handler_state = StatisticsHandlerState::new(&module_config)?;
        let layer_name = LayerName::from("default");
        let layer_config = module_config.layers.get(&layer_name).unwrap();
        let mut telemetry = TelemetryInventoryWithMockedMetrics::new();
        let mut communication = EmptyResultCommunicationInventory::new();
        let mut storage = BlankStorageInventory::new();

        telemetry.response_metrics.expect_iterate_status_codes_responded()
            .with()
            .times(1)
            .returning(|| { vec![StatusCode::OK] });

        telemetry.response_metrics.expect_count_response_by_status_code()
            .with(eq(&StatusCode::OK))
            .times(1)
            .returning(|_| { 5 });

        telemetry.tile_handling_metrics.expect_count_handled_tile_by_source_and_age()
            .with(eq(&TileSource::Cache), eq(&TileAge::Fresh))
            .times(1)
            .returning(|_, _| { 2 });

        telemetry.tile_handling_metrics.expect_count_handled_tile_by_source_and_age()
            .with(eq(&TileSource::Cache), eq(&TileAge::Old))
            .times(1)
            .returning(|_, _| { 1 });

        telemetry.tile_handling_metrics.expect_count_handled_tile_by_source_and_age()
            .with(eq(&TileSource::Cache), eq(&TileAge::VeryOld))
            .times(1)
            .returning(|_, _| { 0 });

        telemetry.tile_handling_metrics.expect_count_handled_tile_by_source_and_age()
            .with(eq(&TileSource::Render), eq(&TileAge::Fresh))
            .times(1)
            .returning(|_, _| { 1 });

        telemetry.tile_handling_metrics.expect_count_handled_tile_by_source_and_age()
            .with(eq(&TileSource::Render), eq(&TileAge::Old))
            .times(1)
            .returning(|_, _| { 1 });

        telemetry.tile_handling_metrics.expect_count_handled_tile_by_source_and_age()
            .with(eq(&TileSource::Render), eq(&TileAge::VeryOld))
            .times(1)
            .returning(|_, _| { 0 });

        telemetry.response_metrics.expect_iterate_valid_zoom_levels()
            .with()
            .times(1)
            .returning(|| { 7..9 });

        telemetry.response_metrics.expect_count_response_by_zoom_level()
            .with(eq(7))
            .times(1)
            .returning(|_| { 2 });

        telemetry.response_metrics.expect_count_response_by_zoom_level()
            .with(eq(8))
            .times(1)
            .returning(|_| { 3 });

        telemetry.response_metrics.expect_count_tile_response_by_zoom_level()
            .with(eq(7))
            .times(1)
            .returning(|_| { 2 });

        telemetry.response_metrics.expect_count_tile_response_by_zoom_level()
            .with(eq(8))
            .times(1)
            .returning(|_| { 3 });

        telemetry.response_metrics.expect_tally_tile_response_duration_by_zoom_level()
            .with(eq(7))
            .times(1)
            .returning(|_| { 1 });

        telemetry.response_metrics.expect_tally_tile_response_duration_by_zoom_level()
            .with(eq(8))
            .times(1)
            .returning(|_| { 2 });

        telemetry.response_metrics.expect_count_total_tile_response()
            .with()
            .times(1)
            .returning(|| { 5 });

        telemetry.response_metrics.expect_tally_total_tile_response_duration()
            .with()
            .times(1)
            .returning(|| { 3 });

        let layer_name_copy1 = layer_name.clone();
        telemetry.response_metrics.expect_iterate_layers_responded()
            .with()
            .times(1)
            .returning(move || { vec![layer_name_copy1] });

        telemetry.response_metrics.expect_count_response_by_layer_and_status_code()
            .with(eq(layer_name.clone()), eq(&StatusCode::OK))
            .times(1)
            .returning(|_, _| { 5 });

        telemetry.response_metrics.expect_count_response_by_layer_and_status_code()
            .with(eq(layer_name.clone()), eq(&StatusCode::NOT_FOUND))
            .times(1)
            .returning(|_, _| { 1 });

        with_request_rec(|request| {
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            request.uri = uri.into_raw();
            let handle_context = HandleContext {
                module_config: &module_config,
                host: VirtualHost::find_or_allocate_new(request)?,
                request: Apache2Request::create_with_tile_config(request)?,
                telemetry: &telemetry,
            };
            let mut io_context = IOContext::new(
                &mut communication,
                &mut storage,
            );
            let request = request::SlippyRequest {
                header: request::Header::new_with_layer(
                    handle_context.request.record,
                    &layer_name,
                ),
                body: request::BodyVariant::ReportStatistics,
            };

            let actual_response = handler_state.handle(&handle_context, &mut io_context, &request).expect_processed().result?;
            let mut expected_data = response::Statistics::new();
            expected_data.number_response_200 = 5;
            expected_data.number_fresh_cache = 2;
            expected_data.number_old_cache = 1;
            expected_data.number_fresh_render = 1;
            expected_data.number_old_render = 1;
            expected_data.number_successful_response_by_zoom[7] = 2;
            expected_data.number_successful_response_by_zoom[8] = 3;
            expected_data.number_tile_response_by_zoom[7] = 2;
            expected_data.number_tile_response_by_zoom[8] = 3;
            expected_data.total_number_tile_response = 5;
            expected_data.total_duration_tile_response = 3;
            expected_data.duration_tile_response_by_zoom[7] = 1;
            expected_data.duration_tile_response_by_zoom[8] = 2;
            expected_data.number_response_200_by_layer.insert(String::from(layer_name.as_str()), 5);
            expected_data.number_response_404_by_layer.insert(String::from(layer_name.as_str()), 1);
            let expected_response = response::SlippyResponse {
                header: response::Header::new(
                    handle_context.request.record,
                    &mime::TEXT_PLAIN,
                ),
                body: response::BodyVariant::Statistics(expected_data),
            };
            assert_eq!(expected_response, actual_response, "Incorrect handling");
            Ok(())
        })
    }
}
