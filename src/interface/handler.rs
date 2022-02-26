use crate::schema::handler::context::HandleContext;
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::request::Request;
use crate::schema::slippy::result::ReadRequestResult;


pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &Request,
    ) -> HandleRequestResult;
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
        obj: &dyn RequestHandler,
        context: &HandleContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
    ) -> ();
}


#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::schema::handler::result::HandleOutcome;
    use crate::schema::slippy::request;
    use chrono::Utc;


    pub struct MockRequestHandler { }

    impl RequestHandler for MockRequestHandler {
        fn handle(
            &mut self,
            _context: &HandleContext,
            _request: &request::Request,
        ) -> HandleRequestResult {
            return HandleRequestResult {
                before_timestamp: Utc::now(),
                after_timestamp: Utc::now(),
                result: Ok(HandleOutcome::NotHandled),
            };
        }
    }
}
