use chrono::{DateTime, TimeZone, Utc,};

use crate::binding::apache2::request_rec;


pub struct Apache2Request<'r> {
    pub record: &'r request_rec,
    pub request_id: i64,
    pub uri: &'r str,
    pub received_timestamp: DateTime<Utc>,
}
