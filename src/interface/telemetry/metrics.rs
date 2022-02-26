use crate::schema::tile::age::TileAge;

use http::status::StatusCode;

pub trait ResponseMetrics {
    fn count_response_by_status_code(&self, status_code: &StatusCode) -> u64;

    fn count_response_by_zoom_level(&self, zoom: u32) -> u64;

    fn count_response_by_status_code_and_zoom_level(&self, status_code: &StatusCode, zoom: u32) -> u64;

    fn count_total_tile_response(&self) -> u64;

    fn tally_total_tile_response_duration(&self) -> u64;

    fn count_tile_response_by_zoom_level(&self, zoom: u32) -> u64;

    fn tally_tile_response_duration_by_zoom_level(&self, zoom: u32) -> u64;
}

pub trait CacheMetrics {
    fn count_tile_cache_hit_by_age(&self, age: &TileAge) -> u64;
}

pub trait RenderMetrics {
    fn count_tile_renders_by_age(&self, age: &TileAge) -> u64;
}


#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::boxed::Box;
    use std::error::Error;
    use std::result::Result;

    pub struct MockZeroResponseMetrics { }

    impl ResponseMetrics for MockZeroResponseMetrics {
        fn count_response_by_status_code(&self, status_code: &StatusCode) -> u64 { 0 }

        fn count_response_by_zoom_level(&self, zoom: u32) -> u64 { 0 }

        fn count_response_by_status_code_and_zoom_level(&self, status_code: &StatusCode, zoom: u32) -> u64 { 0 }

        fn count_total_tile_response(&self) -> u64 { 0 }

        fn tally_total_tile_response_duration(&self) -> u64 { 0 }

        fn count_tile_response_by_zoom_level(&self, zoom: u32) -> u64 { 0 }

        fn tally_tile_response_duration_by_zoom_level(&self, zoom: u32) -> u64 { 0 }
    }

    pub struct MockZeroCacheMetrics { }

    impl CacheMetrics for MockZeroCacheMetrics {
        fn count_tile_cache_hit_by_age(&self, age: &TileAge) -> u64 { 0 }
    }

    pub struct MockZeroRenderMetrics { }

    impl RenderMetrics for MockZeroRenderMetrics {
        fn count_tile_renders_by_age(&self, age: &TileAge) -> u64 { 0 }
    }

    pub fn with_mock_zero_metrics<F>(func: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce(&dyn CacheMetrics, &dyn RenderMetrics, &dyn ResponseMetrics) -> Result<(), Box<dyn Error>> {
        let cache = MockZeroCacheMetrics { };
        let render = MockZeroRenderMetrics { };
        let response = MockZeroResponseMetrics { };
        return func(&cache, &render, &response);
    }
}
