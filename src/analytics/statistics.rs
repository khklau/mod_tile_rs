use crate::analytics::interface::ParseRequestObserver;

use crate::apache2::request::RequestContext;
use crate::slippy::interface::{ ParseRequestFunc, ParseRequestResult };


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
