use crate::analytics::interface::{ HandleRequestObserver, ParseRequestObserver, };

use crate::apache2::request::RequestContext;
use crate::handler::interface::{ HandleRequestResult, RequestHandler, };
use crate::slippy::interface::{ ParseRequestFunc, ParseRequestResult };
use crate::slippy::request::Request;


pub struct ModuleStatistics {}

impl ParseRequestObserver for ModuleStatistics {
    fn on_parse(
        &mut self,
        _func: ParseRequestFunc,
        _context: &RequestContext,
        _url: &str,
        _result: &ParseRequestResult
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
