use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ReadOutcome, WriteOutcome,};
use crate::interface::apache2::HttpResponseWriter;
use crate::interface::handler::{HandleContext, HandleRequestObserver};
use crate::interface::slippy::{
    ReadContext, ReadRequestObserver, WriteContext, WriteResponseObserver,
};


pub struct TransactionTrace {}

impl TransactionTrace {
    pub fn new(_config: &ModuleConfig) -> Result<TransactionTrace, InvalidConfigError> {
        Ok(
            TransactionTrace { }
        )
    }
}

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

    fn on_handle2(
        &mut self,
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

#[cfg(test)]
pub mod test_utils {
    use super::*;


    pub struct NoOpTransactionTrace {}

    impl NoOpTransactionTrace {
        pub fn new() -> NoOpTransactionTrace {
            NoOpTransactionTrace { }
        }
    }

    impl ReadRequestObserver for NoOpTransactionTrace {
        fn on_read(
            &mut self,
            _context: &ReadContext,
            _request: &Apache2Request,
            _read_outcome: &ReadOutcome,
            _read_func_name: &'static str,
        ) -> () {
        }
    }

    impl HandleRequestObserver for NoOpTransactionTrace {
        fn on_handle(
            &mut self,
            _context: &HandleContext,
            _request: &SlippyRequest,
            _handle_outcome: &HandleOutcome,
            _handler_name: &'static str,
            _read_outcome: &ReadOutcome,
        ) -> () {
        }

        fn on_handle2(
            &mut self,
            _request: &SlippyRequest,
            _handle_outcome: &HandleOutcome,
            _handler_name: &'static str,
            _read_outcome: &ReadOutcome,
        ) -> () {
        }
    }

    impl WriteResponseObserver for NoOpTransactionTrace {
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
}
