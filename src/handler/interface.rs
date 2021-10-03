use crate::handler::error::HandleError;

use crate::apache2::request::RequestContext;
use crate::slippy::request::Request;
use crate::slippy::response::Response;


#[derive(Debug)]
pub enum HandleOutcome {
    Handled(Response),
    NotHandled,
}

#[cfg(test)]
impl HandleOutcome {
    pub fn expect_handled(self) -> Response {
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

pub type HandleResult = Result<HandleOutcome, HandleError>;
pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &RequestContext,
        request: &Request,
    ) -> HandleResult;
}