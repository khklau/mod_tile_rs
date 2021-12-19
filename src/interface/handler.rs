use crate::apache2::request::RequestContext;
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::request::Request;
use crate::schema::slippy::result::ReadRequestResult;


pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &RequestContext,
        request: &Request,
    ) -> HandleRequestResult;
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
        obj: &dyn RequestHandler,
        context: &RequestContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
    ) -> ();
}
