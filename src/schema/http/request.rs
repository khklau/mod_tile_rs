use chrono::{DateTime, Utc,};


#[derive(Debug)]
pub struct HttpRequest<'r> {
    pub uri: &'r str,
    pub received_time: DateTime<Utc>,
}
