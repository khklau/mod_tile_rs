use crate::apache2::request::RequestContext;
use crate::handler::interface::{ HandleRequestResult, RequestHandler, };
use crate::slippy::interface::{ ParseRequestFunc, ParseRequestResult, };
use crate::slippy::request::Request;


pub trait ParseRequestObserver {
    fn on_parse(
        &mut self,
        func: ParseRequestFunc,
        context: &RequestContext,
        url: &str,
        result: &ParseRequestResult,
    ) -> ();
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
        obj: &dyn RequestHandler,
        context: &RequestContext,
        request: &Request,
        result: &HandleRequestResult,
    ) -> ();
}
