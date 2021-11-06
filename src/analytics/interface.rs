use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::handler::interface::{ HandleRequestResult, RequestHandler, };
use crate::slippy::interface::{
    ReadRequestFunc, ReadRequestResult, WriteResponseFunc, WriteResponseResult,
};
use crate::slippy::request::Request;
use crate::slippy::response::Response;


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

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        func: WriteResponseFunc,
        context: &ResponseContext,
        response: &Response,
        result: &WriteResponseResult,
    ) -> ();
}
