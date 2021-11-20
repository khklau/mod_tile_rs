use crate::observer::interface::{ HandleRequestObserver, ReadRequestObserver, WriteResponseObserver, };

use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::handler::interface::{ HandleRequestResult, RequestHandler, };
use crate::slippy::interface::{
    ReadRequestFunc, ReadRequestResult, WriteResponseFunc, WriteResponseResult,
};
use crate::schema::slippy::request::Request;
use crate::schema::slippy::response::Response;


pub struct TransactionObserver {}

impl ReadRequestObserver for TransactionObserver {
    fn on_read(
        &mut self,
        _func: ReadRequestFunc,
        _context: &RequestContext,
        _result: &ReadRequestResult
    ) -> () {
    }
}

impl HandleRequestObserver for TransactionObserver {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &RequestContext,
        _request: &Request,
        _result: &HandleRequestResult,
    ) -> () {
    }

}

impl WriteResponseObserver for TransactionObserver {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        _context: &ResponseContext,
        _response: &Response,
        _result: &WriteResponseResult,
    ) -> () {
    }
}
