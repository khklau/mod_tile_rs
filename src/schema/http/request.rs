use crate::binding::apache2::request_rec;

use chrono::{DateTime, Utc,};


#[derive(Debug)]
pub struct HttpRequest<'r> {
    pub uri: &'r str,
    pub received_time: DateTime<Utc>,
    record: &'r request_rec,
}

impl<'r> HttpRequest<'r> {
    pub fn new(
        uri: &'r str,
        received_time: DateTime<Utc>,
        record: &'r request_rec,

    ) -> HttpRequest<'r> {
        HttpRequest {
            uri,
            received_time,
            record,
        }
    }
}
