use http::header::HeaderMap;
use http::status::StatusCode;


#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub bytes_written: usize,
    pub http_headers: HeaderMap,
}
