use crate::analytics::interface::{ HandleRequestObserver, ReadRequestObserver, };

use crate::apache2::request::RequestContext;
use crate::handler::interface::{ HandleRequestResult, RequestHandler, };
use crate::slippy::interface::{ ReadRequestFunc, ReadRequestResult };
use crate::slippy::request::Request;


pub struct ModuleStatistics {}

impl ReadRequestObserver for ModuleStatistics {
    fn on_read(
        &mut self,
        _func: ReadRequestFunc,
        _context: &RequestContext,
        _result: &ReadRequestResult
    ) -> () {
    }
}

impl HandleRequestObserver for ModuleStatistics {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &RequestContext,
        _request: &Request,
        _result: &HandleRequestResult,
    ) -> () {
    }

}
