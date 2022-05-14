use crate::schema::core::processed::ProcessOutcome;
use crate::schema::handler::error::HandleError;
use crate::schema::slippy::response::SlippyResponse;

use chrono::{ DateTime, Utc, };


pub struct HandleRequestResult {
    pub before_timestamp: DateTime<Utc>,
    pub after_timestamp: DateTime<Utc>,
    pub result: Result<SlippyResponse, HandleError>,
}

pub type HandleOutcome = ProcessOutcome<HandleRequestResult>;
