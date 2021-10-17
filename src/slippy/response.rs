use crate::apache2::bindings::{
    conn_rec, request_rec, server_rec,
};

use mime::Mime;
use serde::Serialize;

use std::string::String;


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

#[derive(Debug, PartialEq)]
pub enum BodyVariant {
    Description(Description),
    Tile,
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
