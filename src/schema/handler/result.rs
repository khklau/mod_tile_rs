use crate::schema::handler::error::HandleError;
use crate::schema::slippy::response::SlippyResponse;

use chrono::{ DateTime, Utc, };


#[derive(Debug)]
pub enum HandleOutcome {
    Handled(SlippyResponse),
    NotHandled,
}

#[cfg(test)]
impl HandleOutcome {
    pub fn expect_handled(self) -> SlippyResponse {
        if let HandleOutcome::Handled(response) = self {
            response
        } else {
            panic!("Expected handled HandleOutcome");
        }
    }

    pub fn is_not_handled(self) -> bool {
        if let HandleOutcome::NotHandled = self {
            true
        } else {
            false
        }
    }
}

pub struct HandleRequestResult {
    pub before_timestamp: DateTime<Utc>,
    pub after_timestamp: DateTime<Utc>,
    pub result: Result<HandleOutcome, HandleError>,
}
