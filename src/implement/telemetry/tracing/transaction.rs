use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::schema::apache2::request::Apache2Request;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::slippy::{ ReadContext, WriteContext, };


pub struct TransactionTrace {}

impl ReadRequestObserver for TransactionTrace {
    fn on_read(
        &mut self,
        _func: ReadRequestFunc,
        _context: &ReadContext,
        _request: &Apache2Request,
        _read_outcome: &ReadOutcome,
    ) -> () {
    }
}

impl HandleRequestObserver for TransactionTrace {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &HandleContext,
        _request: &SlippyRequest,
        _handle_outcome: &HandleOutcome,
    ) -> () {
    }

}

impl WriteResponseObserver for TransactionTrace {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        _context: &WriteContext,
        _read_outcome: &ReadOutcome,
        _handle_outcome: &HandleOutcome,
        _write_outcome: &WriteOutcome,
    ) -> () {
    }
}
