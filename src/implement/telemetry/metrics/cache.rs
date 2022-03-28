use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::result::ReadRequestResult;
use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};


pub struct CacheAnalysis {}

impl HandleRequestObserver for CacheAnalysis {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &HandleContext,
        _read_result: &ReadRequestResult,
        _handle_result: &HandleRequestResult,
    ) -> () {
    }

}
