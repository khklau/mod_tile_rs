use crate::binding::apache2::request_rec;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::interface::apache2::PoolStored;

use mime::Mime;
use serde::Serialize;

use std::collections::HashMap;
use std::ffi::CString;
use std::string::String;
use std::vec::Vec;


#[derive(Debug, PartialEq)]
pub struct SlippyResponse {
    pub header: Header,
    pub body: BodyVariant,
}

#[derive(Debug, PartialEq)]
pub struct Header {
    pub host_key: CString,
    pub connection_key: CString,
    pub request_key: CString,
    pub mime_type: Mime,
}

impl Header {
    pub fn new(
        request: &request_rec,
        mime_type: &Mime,
    ) -> Header {
        Header {
            host_key: VirtualHost::search_pool_key(request),
            connection_key: Connection::search_pool_key(request),
            request_key: Apache2Request::search_pool_key(request),
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
