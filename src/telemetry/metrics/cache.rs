use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler };
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::request::Request;
use crate::schema::slippy::response::Response;
use crate::schema::slippy::result::{ ReadRequestResult, WriteResponseResult, };


pub struct CacheAnalysis {}

impl ReadRequestObserver for CacheAnalysis {
    fn on_read(
        &mut self,
        _func: ReadRequestFunc,
        _context: &RequestContext,
        _result: &ReadRequestResult
    ) -> () {
    }
}

impl HandleRequestObserver for CacheAnalysis {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &RequestContext,
        _request: &Request,
        _result: &HandleRequestResult,
    ) -> () {
    }

}

impl WriteResponseObserver for CacheAnalysis {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        _context: &ResponseContext,
        _response: &Response,
        _result: &WriteResponseResult,
    ) -> () {
    }
}
