use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::io::communication::HttpResponseWriter;
use crate::interface::context::{HostContext, RequestContext,};
use crate::adapter::slippy::interface::{ReadRequestObserver, WriteResponseObserver,};
use crate::interface::handler::HandleRequestObserver;


pub struct ReadCounter {
    pub count: u32,
}

impl ReadCounter {
    pub fn new(_config: &ModuleConfig) -> Result<ReadCounter, InvalidConfigError> {
        Ok(
            ReadCounter { count: 0 }
        )
    }
}

impl ReadRequestObserver for ReadCounter {
    fn on_read(
        &mut self,
        _context: &HostContext,
        _request: &Apache2Request,
        _outcome: &ReadOutcome,
        _read_func_name: &'static str,
    ) -> () {
        self.count += 1;
    }
}

pub struct HandleCounter {
    pub count: u32,
}

impl HandleCounter {
    pub fn new(_config: &ModuleConfig) -> Result<HandleCounter, InvalidConfigError> {
        Ok(
            HandleCounter { count: 0 }
        )
    }
}

impl HandleRequestObserver for HandleCounter {
    fn on_handle(
        &mut self,
        _request: &SlippyRequest,
        _handle_outcome: &HandleOutcome,
        _handler_name: &'static str,
    ) -> () {
        self.count += 1;
    }
}

pub struct WriteCounter {
    pub count: u32,
}

impl WriteCounter {
    pub fn new(_config: &ModuleConfig) -> Result<WriteCounter, InvalidConfigError> {
        Ok(
            WriteCounter { count: 0 }
        )
    }
}

impl WriteResponseObserver for WriteCounter {
    fn on_write(
        &mut self,
        _context: &RequestContext,
        _response: &SlippyResponse,
        _writer: &dyn HttpResponseWriter,
        _write_outcome: &WriteOutcome,
        _write_func_name: &'static str,
        _read_outcome: &ReadOutcome,
        _handle_outcome: &HandleOutcome,
    ) -> () {
        self.count += 1;
    }
}
