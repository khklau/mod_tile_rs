use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult };
use crate::schema::slippy::response::{ self, TileResponse };
use crate::schema::slippy::result::{
    ReadRequestResult, ReadOutcome, WriteResponseResult,
};
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::interface::slippy::{
    WriteContext, WriteResponseFunc, WriteResponseObserver,
};
use crate::interface::telemetry::metrics::TileHandlingMetrics;

use enum_iterator::IntoEnumIterator;

use std::collections::hash_map::HashMap;


pub struct TileHandlingAnalysis {
    hit_count_by_age: HashMap<TileAge, u64>,
    render_count_by_age: HashMap<TileAge, u64>,
}

impl TileHandlingAnalysis {
    pub fn new() -> TileHandlingAnalysis {
        TileHandlingAnalysis {
            hit_count_by_age: HashMap::new(),
            render_count_by_age: HashMap::new(),
        }
    }

    fn on_cache_sourced_tile(
        &mut self,
        _context: &WriteContext,
        tile_response: &TileResponse,
    ) -> () {
        let counter = self.hit_count_by_age.entry(tile_response.age).or_insert(0);
        *counter += 1;
    }

    fn on_rendered_tile(
        &mut self,
        _context: &WriteContext,
        tile_response: &TileResponse,
    ) -> () {
        let counter = self.render_count_by_age.entry(tile_response.age).or_insert(0);
        *counter += 1;
    }
}

impl WriteResponseObserver for TileHandlingAnalysis {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        context: &WriteContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
        _write_result: &WriteResponseResult,
    ) -> () {
        match (read_result, &handle_result.result) {
            (Ok(read_outcome), Ok(handle_outcome)) => match (read_outcome, handle_outcome) {
                (ReadOutcome::Matched(_), HandleOutcome::Handled(response)) => match &response.body {
                    response::BodyVariant::Tile(tile_response) => match &tile_response.source {
                        TileSource::Cache => self.on_cache_sourced_tile(context, tile_response),
                        TileSource::Render => self.on_rendered_tile(context, tile_response),
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => ()
        }
    }
}

impl TileHandlingMetrics for TileHandlingAnalysis {
    fn iterate_valid_cache_ages(&self) -> Box<dyn Iterator<Item = TileAge>> {
        Box::new(TileAge::into_enum_iter())
    }

    fn count_tile_cache_hit_by_age(&self, age: &TileAge) -> u64 {
        match self.hit_count_by_age.get(age) {
            Some(count) => *count,
            None => 0
        }
    }

    fn iterate_valid_render_ages(&self) -> Box<dyn Iterator<Item = TileAge>> {
        Box::new(TileAge::into_enum_iter())
    }

    fn count_tile_renders_by_age(&self, age: &TileAge) -> u64 {
        match self.render_count_by_age.get(age) {
            Some(count) => *count,
            None => 0
        }
    }
}