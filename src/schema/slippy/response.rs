use crate::schema::apache2::config::MAX_ZOOM_SERVER;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::interface::tile::TileRef;

use mime::Mime;
use serde::Serialize;

use std::collections::HashMap;
use std::clone::Clone;
use std::default::Default;
use std::string::String;
use std::vec::Vec;


#[derive(Clone, Debug, PartialEq)]
pub struct SlippyResponse {
    pub header: Header,
    pub body: BodyVariant,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub mime_type: Mime,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum BodyVariant {
    Description(Description),
    Statistics(Statistics),
    Tile(TileResponse),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Description {
    pub tilejson: &'static str,
    pub schema: &'static str,
    pub name: String,
    pub description: String,
    pub attribution: String,
    pub minzoom: u64,
    pub maxzoom: u64,
    pub tiles: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Statistics {
    pub number_response_200: u64,
    pub number_response_304: u64,
    pub number_response_404: u64,
    pub number_response_503: u64,
    pub number_response_5xx: u64,
    pub number_response_other: u64,
    pub number_fresh_cache: u64,
    pub number_old_cache: u64,
    pub number_very_old_cache: u64,
    pub number_fresh_render: u64,
    pub number_old_render: u64,
    pub number_very_old_render: u64,
    pub number_successful_response_by_zoom: Vec<u64>,
    pub total_number_tile_response: u64,
    pub total_duration_tile_response: u64,
    pub number_tile_response_by_zoom: Vec<u64>,
    pub duration_tile_response_by_zoom: Vec<u64>,
    pub number_response_200_by_layer: HashMap<String, u64>,
    pub number_response_404_by_layer: HashMap<String, u64>,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            number_response_200: 0,
            number_response_304: 0,
            number_response_404: 0,
            number_response_503: 0,
            number_response_5xx: 0,
            number_response_other: 0,
            number_fresh_cache: 0,
            number_old_cache: 0,
            number_very_old_cache: 0,
            number_fresh_render: 0,
            number_old_render: 0,
            number_very_old_render: 0,
            number_successful_response_by_zoom: vec![Default::default(); MAX_ZOOM_SERVER + 1],
            total_number_tile_response: 0,
            total_duration_tile_response: 0,
            number_tile_response_by_zoom: vec![Default::default(); MAX_ZOOM_SERVER + 1],
            duration_tile_response_by_zoom: vec![Default::default(); MAX_ZOOM_SERVER + 1],
            number_response_200_by_layer: HashMap::new(),
            number_response_404_by_layer:  HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TileResponse {
    pub source: TileSource,
    pub age: TileAge,
    pub tile_ref: TileRef,
}
