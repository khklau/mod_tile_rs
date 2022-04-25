use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult };
use crate::schema::slippy::response::{self, TileResponse};
use crate::schema::slippy::result::{
    ReadRequestResult, ReadOutcome,
};
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};

use std::collections::hash_map::HashMap;


pub struct CacheAnalysis {
    hit_count_by_age: HashMap<TileAge, u64>,
}

impl CacheAnalysis {
    pub fn new() -> CacheAnalysis {
        CacheAnalysis {
            hit_count_by_age: HashMap::new(),
        }
    }

    fn on_cache_sourced_tile(
        &mut self,
        _context: &HandleContext,
        tile_response: &TileResponse,
    ) -> () {
        let counter = self.hit_count_by_age.entry(tile_response.age).or_insert(0);
        *counter += 1;
    }
}

impl HandleRequestObserver for CacheAnalysis {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        context: &HandleContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
    ) -> () {
        match (read_result, &handle_result.result) {
            (Ok(read_outcome), Ok(handle_outcome)) => match (read_outcome, handle_outcome) {
                (ReadOutcome::Matched(_), HandleOutcome::Handled(response)) => match &response.body {
                    response::BodyVariant::Tile(tile_response) => match tile_response.source {
                        TileSource::Cache => self.on_cache_sourced_tile(context, tile_response),
                        _ => (),
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => ()
        }
    }

}
