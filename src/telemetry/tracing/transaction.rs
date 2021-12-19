use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler };
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::result::{ ReadRequestResult, WriteResponseResult, };


pub struct TransactionTrace {}

impl ReadRequestObserver for TransactionTrace {
    fn on_read(
        &mut self,
        _func: ReadRequestFunc,
        _context: &RequestContext,
        _result: &ReadRequestResult
    ) -> () {
    }
}

impl HandleRequestObserver for TransactionTrace {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &RequestContext,
        _read_result: &ReadRequestResult,
        _handle_result: &HandleRequestResult,
    ) -> () {
    }

}

impl WriteResponseObserver for TransactionTrace {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        _context: &ResponseContext,
        _read_result: &ReadRequestResult,
        _handle_result: &HandleRequestResult,
        _write_result: &WriteResponseResult,
    ) -> () {
    }
}
