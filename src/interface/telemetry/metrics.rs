use crate::schema::tile::age::Age;

use http::status::StatusCode;

pub trait ResponseMetrics {
    fn count_response_by_status_code(self, status_code: &StatusCode) -> u64;

    fn count_response_by_zoom_level(self, zoom: u32) -> u64;

    fn count_response_by_status_code_and_zoom_level(self, status_code: &StatusCode, zoom: u32) -> u64;

    fn count_total_tile_response(self) -> u64;

    fn tally_total_tile_response_duration(self) -> u64;

    fn count_tile_response_by_zoom_level(self, zoom: u32) -> u64;

    fn tally_tile_response_duration_by_zoom_level(self, zoom: u32) -> u64;
}

pub trait CacheMetrics {
    fn count_tile_cache_hit_by_age(self, age: &Age) -> u64;
}

pub trait RenderMetrics {
    fn count_tile_renders_by_age(self, age: &Age) -> u64;
}

