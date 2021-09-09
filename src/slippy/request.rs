use std::option::Option;
use std::string::String;


#[derive(PartialEq)]
#[derive(Debug)]
pub enum Request {
    ReportModStats,
    DescribeLayer(DescribeLayerRequest),
    ServeTileV3(ServeTileRequestV3),
    ServeTileV2(ServeTileRequestV2),
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct DescribeLayerRequest {
    pub layer: String,
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
