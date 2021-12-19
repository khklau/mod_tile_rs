use crate::apache2::request::RequestContext;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler };
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::result::ReadRequestResult;


pub struct CacheAnalysis {}

impl HandleRequestObserver for CacheAnalysis {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &RequestContext,
        _read_result: &ReadRequestResult,
        _handle_result: &HandleRequestResult,
    ) -> () {
    }

}
