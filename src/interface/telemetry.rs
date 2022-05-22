use crate::schema::tile::age::TileAge;
use crate::schema::tile::identity::LayerName;
use crate::schema::tile::source::TileSource;

use http::status::StatusCode;

use std::boxed::Box;
use std::iter::Iterator;
use std::ops::Range;


pub trait ResponseMetrics {
    fn iterate_status_codes_responded(&self) -> Box<dyn Iterator<Item = &'_ StatusCode> + '_>;

    fn iterate_valid_zoom_levels(&self) -> Range<u32>;

    fn iterate_layers_responded(&self) -> Box<dyn Iterator<Item = &'_ LayerName> + '_>;

    fn count_response_by_status_code(&self, status_code: &StatusCode) -> u64;

    fn count_response_by_zoom_level(&self, zoom: u32) -> u64;

    fn count_response_by_status_code_and_zoom_level(&self, status_code: &StatusCode, zoom: u32) -> u64;

    fn count_total_tile_response(&self) -> u64;

    fn tally_total_tile_response_duration(&self) -> u64;

    fn count_tile_response_by_zoom_level(&self, zoom: u32) -> u64;

    fn tally_tile_response_duration_by_zoom_level(&self, zoom: u32) -> u64;

    fn count_response_by_layer_and_status_code(&self, layer: &LayerName, status_code: &StatusCode) -> u64;
}

pub trait TileHandlingMetrics {
    fn iterate_valid_cache_ages(&self) -> Box<dyn Iterator<Item = TileAge>>;

    fn iterate_valid_render_ages(&self) -> Box<dyn Iterator<Item = TileAge>>;

    fn count_handled_tile_by_source_and_age(&self, source: &TileSource, age: &TileAge) -> u64;

    fn tally_tile_handle_duration_by_source_and_age(&self, source: &TileSource, age: &TileAge) -> u64;
}

pub struct MetricsInventory<'i> {
    pub response_metrics: &'i dyn ResponseMetrics,
    pub tile_handling_metrics: &'i dyn TileHandlingMetrics,
}


#[cfg(test)]
pub mod test_utils {
    use enum_iterator::IntoEnumIterator;

    use super::*;
    use std::boxed::Box;
    use std::error::Error;
    use std::result::Result;

    pub struct MockZeroResponseMetrics {
        pub mock_status_codes: Vec<StatusCode>,
        pub mock_zoom_levels: Vec<u32>,
        pub mock_layers: Vec<LayerName>,
    }

    impl MockZeroResponseMetrics {
        fn new() -> MockZeroResponseMetrics {
            MockZeroResponseMetrics {
                mock_status_codes: Vec::new(),
                mock_zoom_levels: Vec::new(),
                mock_layers: Vec::new(),
            }
        }
    }

    impl ResponseMetrics for MockZeroResponseMetrics {
        fn iterate_status_codes_responded(&self) -> Box<dyn Iterator<Item = &'_ StatusCode> + '_> {
            Box::new(self.mock_status_codes.iter())
        }

        fn iterate_valid_zoom_levels(&self) -> Range<u32> {
            Range {
                start: 0,
                end: 1,
            }
        }

        fn iterate_layers_responded(&self) -> Box<dyn Iterator<Item = &'_ LayerName> + '_> {
            Box::new(self.mock_layers.iter())
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

    pub struct MockZeroTileHandlingMetrics { }

    impl TileHandlingMetrics for MockZeroTileHandlingMetrics {
        fn iterate_valid_cache_ages(&self) -> Box<dyn Iterator<Item = TileAge>> {
            Box::new(TileAge::into_enum_iter())
        }

        fn iterate_valid_render_ages(&self) -> Box<dyn Iterator<Item = TileAge>> {
            Box::new(TileAge::into_enum_iter())
        }

        fn count_handled_tile_by_source_and_age(&self, _source: &TileSource, _age: &TileAge) -> u64 { 0 }

        fn tally_tile_handle_duration_by_source_and_age(&self, _source: &TileSource, _age: &TileAge) -> u64 { 0 }
    }

    pub fn with_mock_zero_metrics<F>(func: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce(&dyn ResponseMetrics, &dyn TileHandlingMetrics) -> Result<(), Box<dyn Error>> {
        let response = MockZeroResponseMetrics::new();
        let tile = MockZeroTileHandlingMetrics { };
        return func(&response, &tile);
    }
}
