use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::error::HandleError;
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::framework::apache2::context::HostContext;
use crate::service::interface::ServicesContext;

use chrono::Utc;
use http::status::StatusCode;
use mime;

use std::any::type_name;


pub struct StatisticsContext<'c> {
    pub host: HostContext<'c>,
    pub services: ServicesContext<'c>,
}

impl<'c> StatisticsContext<'c> {
    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host.host
    }
}


pub struct StatisticsHandlerState { }

impl StatisticsHandlerState {
    pub fn new(_config: &ModuleConfig) -> Result<StatisticsHandlerState, InvalidConfigError> {
        Ok(
            StatisticsHandlerState {  }
        )
    }

    pub fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    pub fn report_statistics(
        &self,
        context: &StatisticsContext,
        _header: &request::Header,
    ) -> Result<response::SlippyResponse, HandleError> {
        let before_timestamp = Utc::now();
        let statistics = self.report(&context.services);
        let after_timestamp = Utc::now();
        let response = response::SlippyResponse {
            header: response::Header {
                mime_type: mime::TEXT_PLAIN.clone(),
                before_timestamp,
                after_timestamp,
            },
            body: response::BodyVariant::Statistics(statistics),
        };
        return Ok(response);
    }

    fn report(
        &self,
        services: &ServicesContext,
    ) -> response::Statistics {
        let mut result = response::Statistics::new();
        let response_metrics = services.telemetry.response_metrics();
        let tile_handling_metrics = services.telemetry.tile_handling_metrics();
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::identifier::generate_id;
    use crate::schema::tile::identity::LayerName;
    use crate::io::communication::interface::test_utils::EmptyResultCommunicationInventory;
    use crate::use_case::interface::{
        DescriptionUseCaseObserver,
        StatisticsUseCaseObserver,
        TileUseCaseObserver,
    };
    use crate::use_case::interface::test_utils::NoOpHandleRequestObserver;
    use crate::adapter::slippy::interface::{ReadRequestObserver, WriteResponseObserver,};
    use crate::adapter::slippy::interface::test_utils::{NoOpReadRequestObserver, NoOpWriteResponseObserver,};
    use crate::io::storage::interface::test_utils::BlankStorageInventory;
    use crate::service::telemetry::interface::{
        MockResponseMetrics, MockTileHandlingMetrics,
        ResponseMetrics, TileHandlingMetrics, TelemetryInventory
    };
    use crate::service::rendering::interface::test_utils::NoOpRenderingInventory;
    use crate::service::telemetry::interface::test_utils::NoOpZeroTelemetryInventory;
    use crate::framework::apache2::record::test_utils::with_request_rec;

    use chrono::Utc;
    use mockall::predicate::eq;

    use std::error::Error as StdError;
    use std::ffi::CString;

    pub struct TelemetryInventoryWithMockedMetrics {
        response_metrics: MockResponseMetrics,
        tile_handling_metrics: MockTileHandlingMetrics,
        read_observer_0: NoOpReadRequestObserver,
        read_observer_1: NoOpReadRequestObserver,
        handle_observer_0: NoOpHandleRequestObserver,
        handle_observer_1: NoOpHandleRequestObserver,
        description_use_case_observer_0: NoOpHandleRequestObserver,
        description_use_case_observer_1: NoOpHandleRequestObserver,
        statistics_use_case_observer_0: NoOpHandleRequestObserver,
        statistics_use_case_observer_1: NoOpHandleRequestObserver,
        tile_use_case_observer_0: NoOpHandleRequestObserver,
        tile_use_case_observer_1: NoOpHandleRequestObserver,
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
                description_use_case_observer_0: NoOpHandleRequestObserver::new(),
                description_use_case_observer_1: NoOpHandleRequestObserver::new(),
                statistics_use_case_observer_0: NoOpHandleRequestObserver::new(),
                statistics_use_case_observer_1: NoOpHandleRequestObserver::new(),
                tile_use_case_observer_0: NoOpHandleRequestObserver::new(),
                tile_use_case_observer_1: NoOpHandleRequestObserver::new(),
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

        fn description_use_case_observers(&mut self) -> [&mut dyn DescriptionUseCaseObserver; 2] {
            [&mut self.description_use_case_observer_0, &mut self.description_use_case_observer_1]
        }

        fn statistics_use_case_observers(&mut self) -> [&mut dyn StatisticsUseCaseObserver; 2] {
            [&mut self.statistics_use_case_observer_0, &mut self.statistics_use_case_observer_1]
        }

        fn tile_use_case_observers(&mut self) -> [&mut dyn TileUseCaseObserver; 2] {
            [&mut self.tile_use_case_observer_0, &mut self.tile_use_case_observer_1]
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
    fn test_handle_after_tile_render() -> Result<(), Box<dyn StdError>> {
        let module_config = ModuleConfig::new();
        let mut handler_state = StatisticsHandlerState::new(&module_config)?;
        let layer_name = LayerName::from("default");
        let layer_config = module_config.layers.get(&layer_name).unwrap();
        let mut telemetry = TelemetryInventoryWithMockedMetrics::new();
        let mut rendering = NoOpRenderingInventory::new();
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

        with_request_rec(|record| {
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = StatisticsContext {
                host: HostContext::new(&module_config, record),
                services: ServicesContext {
                    telemetry: &telemetry,
                    rendering: &mut rendering,
                },
            };
            let header = request::Header {
                layer: layer_name.clone(),
                request_id: generate_id(),
                uri: uri.into_string()?,
                received_timestamp: Utc::now(),
            };
            let actual_response = handler_state.report_statistics(
                &context,
                &header
            )?;
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
                header: response::Header {
                    mime_type: mime::TEXT_PLAIN.clone(),
                    before_timestamp: actual_response.header.before_timestamp, // Don't care
                    after_timestamp: actual_response.header.after_timestamp, // Don't care
                },
                body: response::BodyVariant::Statistics(expected_data),
            };
            assert_eq!(expected_response, actual_response, "Incorrect handling");
            Ok(())
        })
    }
}
