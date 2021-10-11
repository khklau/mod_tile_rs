use crate::apache2::request::RequestContext;
use crate::handler::interface::{ HandleRequestResult, RequestHandler, };
use crate::slippy::interface::{ ReadRequestFunc, ReadRequestResult, };
use crate::slippy::request::Request;


pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        func: ReadRequestFunc,
        context: &RequestContext,
        result: &ReadRequestResult,
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
