use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
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
