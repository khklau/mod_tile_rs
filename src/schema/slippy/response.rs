use crate::binding::apache2::{
    conn_rec, request_rec, server_rec,
};
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;

use mime::Mime;
use serde::Serialize;

use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;


#[derive(Debug, PartialEq)]
pub struct Response {
    pub header: Header,
    pub body: BodyVariant,
}

#[derive(Debug, PartialEq)]
pub struct Header {
    pub host_id: usize,
    pub request_id: usize,
    pub connection_id: i64,
    pub mime_type: Mime,
}

impl Header {
    pub fn new(
        request: &request_rec,
        connection: &conn_rec,
        host: &server_rec,
        mime_type: &Mime,
    ) -> Header {
        let host_ptr = host as *const server_rec;
        let request_ptr = request as *const request_rec;
        Header {
            host_id: host_ptr as usize,
            request_id: request_ptr as usize,
            connection_id: connection.id,
            mime_type: mime_type.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub enum BodyVariant {
    Description(Description),
    Tile(TileResponse),
}

#[derive(Debug, PartialEq, Serialize)]
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

#[derive(Debug, PartialEq, Serialize)]
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
    pub number_response_zoom: Vec<u64>,
    pub number_tile_buffer_reads: u64,
    pub duration_tile_buffer_reads: u64,
    pub number_tile_buffer_read_zoom: Vec<u64>,
    pub duration_tile_buffer_read_zoom: Vec<u64>,
    pub number_response_200_by_layer: HashMap<String, u64>,
    pub number_response_400_by_layer: HashMap<String, u64>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct TileResponse {
    pub source: TileSource,
    pub age: TileAge,
}
