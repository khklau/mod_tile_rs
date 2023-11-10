use crate::binding::apache2::request_rec;
use crate::schema::tile::identity::LayerName;

use std::option::Option;
use std::string::String;


pub const MAX_EXTENSION_LEN: usize = crate::schema::renderd::request::MAX_MIME_TYPE_LEN;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct SlippyRequest {
    pub header: Header,
    pub body: BodyVariant,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct Header {
    pub layer: LayerName,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum BodyVariant {
    ReportStatistics,
    DescribeLayer,
    ServeTileV3(ServeTileRequestV3),
    ServeTileV2(ServeTileRequestV2),
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct ServeTileRequestV3 {
    pub parameter: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub extension: String,
    pub option: Option<String>,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct ServeTileRequestV2 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub extension: String,
    pub option: Option<String>,
}
