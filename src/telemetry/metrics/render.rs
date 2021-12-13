use crate::apache2::request::RequestContext;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler };
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::request::Request;


pub struct RenderAnalysis {}

impl HandleRequestObserver for RenderAnalysis {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &RequestContext,
        _request: &Request,
        _result: &HandleRequestResult,
    ) -> () {
    }

}
