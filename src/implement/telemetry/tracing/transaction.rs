use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::result::{ ReadOutcome, WriteResponseResult, };
use crate::interface::slippy::{ ReadContext, WriteContext, };


pub struct TransactionTrace {}

impl ReadRequestObserver for TransactionTrace {
    fn on_read(
        &mut self,
        _func: ReadRequestFunc,
        _context: &ReadContext,
        _outcome: &ReadOutcome,
    ) -> () {
    }
}

impl HandleRequestObserver for TransactionTrace {
    fn on_handle(
        &mut self,
        _obj: &dyn RequestHandler,
        _context: &HandleContext,
        _read_outcome: &ReadOutcome,
        _handle_result: &HandleRequestResult,
    ) -> () {
    }

}

impl WriteResponseObserver for TransactionTrace {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        _context: &WriteContext,
        _read_outcome: &ReadOutcome,
        _handle_result: &HandleRequestResult,
        _write_result: &WriteResponseResult,
    ) -> () {
    }
}
