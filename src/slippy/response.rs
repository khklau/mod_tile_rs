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
    pub mime_type: &'static str,
}

impl Header {
    pub fn new(
        request: &request_rec,
        connection: &conn_rec,
        host: &server_rec,
        mime_type: &'static str,
    ) -> Header {
        let host_ptr = host as *const server_rec;
        let request_ptr = request as *const request_rec;
        Header {
            host_id: host_ptr as usize,
            request_id: request_ptr as usize,
            connection_id: connection.id,
            mime_type: mime_type,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BodyVariant {
    Text(String),
    Tile,
}
