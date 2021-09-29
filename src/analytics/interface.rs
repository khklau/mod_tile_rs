use crate::apache2::request::RequestContext;
use crate::slippy::interface::{ ParseRequestFunc, ParseRequestResult };


pub trait ParseRequestObserver {
    fn on_parse(
        &mut self,
        func: ParseRequestFunc,
        context: &RequestContext,
        url: &str,
        result: &ParseRequestResult
    ) -> ();
}
