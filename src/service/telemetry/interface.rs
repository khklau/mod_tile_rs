use crate::schema::tile::age::TileAge;
use crate::schema::tile::identity::LayerName;
use crate::schema::tile::source::TileSource;
use crate::use_case::interface::{
    DescriptionUseCaseObserver,
    StatisticsUseCaseObserver,
    TileUseCaseObserver,
};
use crate::adapter::slippy::interface::{ReadRequestObserver, WriteResponseObserver,};

use http::status::StatusCode;
#[cfg(test)]
use mockall::{automock, mock, predicate::*};

use std::ops::Range;
use std::vec::Vec;


#[cfg_attr(test, automock)]
pub trait ResponseMetrics {
    fn iterate_status_codes_responded(&self) -> Vec<StatusCode>;

    fn iterate_valid_zoom_levels(&self) -> Range<u32>;

    fn iterate_layers_responded(&self) -> Vec<LayerName>;

    fn count_response_by_status_code(&self, status_code: &StatusCode) -> u64;

    fn count_response_by_zoom_level(&self, zoom: u32) -> u64;

    fn count_response_by_status_code_and_zoom_level(&self, status_code: &StatusCode, zoom: u32) -> u64;

    fn count_total_tile_response(&self) -> u64;

    fn tally_total_tile_response_duration(&self) -> u64;

    fn count_tile_response_by_zoom_level(&self, zoom: u32) -> u64;

    fn tally_tile_response_duration_by_zoom_level(&self, zoom: u32) -> u64;

    fn count_response_by_layer_and_status_code(&self, layer: &LayerName, status_code: &StatusCode) -> u64;
}

#[cfg_attr(test, automock)]
pub trait TileHandlingMetrics {
    fn iterate_valid_cache_ages(&self) -> Vec<TileAge>;

    fn iterate_valid_render_ages(&self) -> Vec<TileAge>;

    fn count_handled_tile_by_source_and_age(&self, source: &TileSource, age: &TileAge) -> u64;

    fn tally_tile_handle_duration_by_source_and_age(&self, source: &TileSource, age: &TileAge) -> u64;
}

pub trait TelemetryInventory {
    fn response_metrics(&self) -> &dyn ResponseMetrics;
    // TODO: add a method that returns the concrete type name

    fn tile_handling_metrics(&self) -> &dyn TileHandlingMetrics;
    // TODO: add a method that returns the concrete type name

    fn read_request_observers(&mut self) -> [&mut dyn ReadRequestObserver; 2];

    fn description_use_case_observers(&mut self) -> [&mut dyn DescriptionUseCaseObserver; 2];

    fn statistics_use_case_observers(&mut self) -> [&mut dyn StatisticsUseCaseObserver; 2];

    fn tile_use_case_observers(&mut self) -> [&mut dyn TileUseCaseObserver; 2];

    fn write_response_observers(&mut self) -> [&mut dyn WriteResponseObserver; 4];
}


#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::use_case::interface::test_utils::NoOpHandleRequestObserver;
    use crate::adapter::slippy::interface::test_utils::{NoOpReadRequestObserver, NoOpWriteResponseObserver,};

    use enum_iterator::IntoEnumIterator;

    pub struct ZeroResponseMetrics {
        pub mock_status_codes: Vec<StatusCode>,
        pub mock_layers: Vec<LayerName>,
    }

    impl ZeroResponseMetrics {
        fn new() -> ZeroResponseMetrics {
            ZeroResponseMetrics {
                mock_status_codes: Vec::new(),
                mock_layers: Vec::new(),
            }
        }
    }

    impl ResponseMetrics for ZeroResponseMetrics {
        fn iterate_status_codes_responded(&self) -> Vec<StatusCode> {
            self.mock_status_codes.iter().cloned().collect()
        }

        fn iterate_valid_zoom_levels(&self) -> Range<u32> {
            Range {
                start: 0,
                end: 1,
            }
        }

        fn iterate_layers_responded(&self) -> Vec<LayerName> {
            self.mock_layers.iter().cloned().collect()
        }

        fn count_response_by_status_code(&self, _status_code: &StatusCode) -> u64 { 0 }

        fn count_response_by_zoom_level(&self, _zoom: u32) -> u64 { 0 }

        fn count_response_by_status_code_and_zoom_level(&self, _status_code: &StatusCode, _zoom: u32) -> u64 { 0 }

        fn count_total_tile_response(&self) -> u64 { 0 }

        fn tally_total_tile_response_duration(&self) -> u64 { 0 }

        fn count_tile_response_by_zoom_level(&self, _zoom: u32) -> u64 { 0 }

        fn tally_tile_response_duration_by_zoom_level(&self, _zoom: u32) -> u64 { 0 }

        fn count_response_by_layer_and_status_code(&self, _layer: &LayerName, _status_code: &StatusCode) -> u64 { 0 }
    }

    pub struct ZeroTileHandlingMetrics { }

    impl ZeroTileHandlingMetrics {
        pub fn new() -> ZeroTileHandlingMetrics {
            ZeroTileHandlingMetrics { }
        }
    }

    impl TileHandlingMetrics for ZeroTileHandlingMetrics {
        fn iterate_valid_cache_ages(&self) -> Vec<TileAge> {
            TileAge::into_enum_iter().collect()
        }

        fn iterate_valid_render_ages(&self) -> Vec<TileAge> {
            TileAge::into_enum_iter().collect()
        }

        fn count_handled_tile_by_source_and_age(&self, _source: &TileSource, _age: &TileAge) -> u64 { 0 }

        fn tally_tile_handle_duration_by_source_and_age(&self, _source: &TileSource, _age: &TileAge) -> u64 { 0 }
    }

    pub struct NoOpZeroTelemetryInventory {
        response_metrics: ZeroResponseMetrics,
        tile_handling_metrics: ZeroTileHandlingMetrics,
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

    impl NoOpZeroTelemetryInventory {
        pub fn new() -> NoOpZeroTelemetryInventory {
            NoOpZeroTelemetryInventory {
                response_metrics: ZeroResponseMetrics::new(),
                tile_handling_metrics: ZeroTileHandlingMetrics::new(),
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

    impl TelemetryInventory for NoOpZeroTelemetryInventory {
        fn response_metrics(&self) -> &dyn ResponseMetrics {
            &self.response_metrics
        }

        fn tile_handling_metrics(&self) -> &dyn TileHandlingMetrics {
            &self.tile_handling_metrics
        }

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
}
