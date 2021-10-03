use crate::apache2::bindings::{
    conn_rec, request_rec, server_rec,
};

use std::string::String;


#[derive(PartialEq)]
#[derive(Debug)]
pub struct Response {
    pub header: Header,
    pub body: BodyVariant,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct Header {
    pub host_id: usize,
    pub request_id: usize,
    pub connection_id: i64,
    pub layer: String,
}

impl Header {
    pub fn new(
        request: &request_rec,
        connection: &conn_rec,
        host: &server_rec,
    ) -> Header {
        let layer = String::new();
        Self::new_with_layer(request, connection, host, &layer)
    }

    pub fn new_with_layer(
        request: &request_rec,
        connection: &conn_rec,
        host: &server_rec,
        layer: &String,
    ) -> Header {
        let host_ptr = host as *const server_rec;
        let request_ptr = request as *const request_rec;
        Header {
            host_id: host_ptr as usize,
            request_id: request_ptr as usize,
            connection_id: connection.id,
            layer: layer.clone(),
        }
    }
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum BodyVariant {
    StatisticsReport,
    LayerDescription(String),
    Tile,
}
