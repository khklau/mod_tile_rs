use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::schema::apache2::request::Apache2Request;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::apache2::HttpResponseWriter;
use crate::interface::slippy::{ ReadContext, WriteContext, };


pub struct TransactionTrace {}

impl ReadRequestObserver for TransactionTrace {
    fn on_read(
        &mut self,
        _context: &ReadContext,
        _request: &Apache2Request,
        _read_outcome: &ReadOutcome,
        _read_func_name: &'static str,
    ) -> () {
    }
}

impl HandleRequestObserver for TransactionTrace {
    fn on_handle(
        &mut self,
        _context: &HandleContext,
        _request: &SlippyRequest,
        _handle_outcome: &HandleOutcome,
        _handler_name: &'static str,
        _read_outcome: &ReadOutcome,
    ) -> () {
    }
}

impl WriteResponseObserver for TransactionTrace {
    fn on_write(
        &mut self,
        _context: &WriteContext,
        _response: &SlippyResponse,
        _writer: &dyn HttpResponseWriter,
        _write_outcome: &WriteOutcome,
        _write_func_name: &'static str,
        _read_outcome: &ReadOutcome,
        _handle_outcome: &HandleOutcome,
    ) -> () {
    }
}
