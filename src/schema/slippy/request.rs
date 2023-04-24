use crate::binding::apache2::request_rec;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::tile::identity::LayerName;
use crate::interface::apache2::PoolStored;

use std::ffi::CString;
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
    pub host_key: CString,
    pub connection_key: CString,
    pub request_key: CString,
    pub layer: LayerName,
}

impl Header {
    pub fn new(
        request: &request_rec,
    ) -> Header {
        let layer = LayerName::new();
        Self::new_with_layer(request, &layer)
    }

    pub fn new_with_layer(
        request: &request_rec,
        layer: &LayerName,
    ) -> Header {
        Header {
            host_key: VirtualHost::search_pool_key(request),
            connection_key: Connection::search_pool_key(request),
            request_key: Apache2Request::search_pool_key(request),
            layer: layer.clone(),
        }
    }
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
