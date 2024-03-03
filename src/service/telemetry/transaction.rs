use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::http::request::HttpRequest;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ReadOutcome, WriteOutcome,};
use crate::io::communication::interface::HttpResponseWriter;
use crate::adapter::slippy::interface::{
    ReadContext,
    ReadRequestObserver,
    WriteContext,
    WriteResponseObserver,
};
use crate::use_case::interface::{
    DescriptionUseCaseObserver,
    StatisticsUseCaseObserver,
    TileUseCaseObserver,
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
        _request: &HttpRequest,
        _read_outcome: &ReadOutcome,
        _read_func_name: &'static str,
    ) -> () {
    }
}

impl DescriptionUseCaseObserver for TransactionTrace {
    fn on_describe_layer(
        &mut self,
        _request: &SlippyRequest,
        _handle_outcome: &HandleOutcome,
        _handler_name: &'static str,
    ) -> () {
    }
}

impl StatisticsUseCaseObserver for TransactionTrace {
    fn on_report_statistics(
        &mut self,
        _request: &SlippyRequest,
        _handle_outcome: &HandleOutcome,
        _handler_name: &'static str,
    ) -> () {
    }
}

impl TileUseCaseObserver for TransactionTrace {
    fn on_fetch_tile(
        &mut self,
        _request: &SlippyRequest,
        _handle_outcome: &HandleOutcome,
        _handler_name: &'static str,
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
