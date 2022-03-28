use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::result::ReadRequestResult;
use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};


pub struct RenderAnalysis {}

impl HandleRequestObserver for RenderAnalysis {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &HandleContext,
        _read_result: &ReadRequestResult,
        _handle_result: &HandleRequestResult,
    ) -> () {
    }

}
