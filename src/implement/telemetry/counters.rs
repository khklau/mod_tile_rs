use crate::schema::apache2::request::Apache2Request;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::apache2::HttpResponseWriter;
use crate::interface::slippy::{
    ReadContext, ReadRequestObserver, WriteContext, WriteResponseObserver};
use crate::interface::handler::{HandleContext, HandleRequestObserver};


pub struct CountingReadObserver {
    pub count: u32,
}

impl CountingReadObserver {
    pub fn new() -> CountingReadObserver {
        CountingReadObserver { count: 0 }
    }
}

impl ReadRequestObserver for CountingReadObserver {
    fn on_read(
        &mut self,
        _context: &ReadContext,
        _request: &Apache2Request,
        _outcome: &ReadOutcome,
        _read_func_name: &'static str,
    ) -> () {
        self.count += 1;
    }
}

pub struct CountingHandleObserver {
    pub count: u32,
}

impl CountingHandleObserver {
    pub fn new() -> CountingHandleObserver {
        CountingHandleObserver { count: 0 }
    }
}

impl HandleRequestObserver for CountingHandleObserver {
    fn on_handle(
        &mut self,
        _context: &HandleContext,
        _request: &SlippyRequest,
        _handle_outcome: &HandleOutcome,
        _handler_name: &'static str,
        _read_outcome: &ReadOutcome,
    ) -> () {
        self.count += 1;
    }
}

pub struct CountingWriteObserver {
    pub count: u32,
}

impl CountingWriteObserver {
    pub fn new() -> CountingWriteObserver {
        CountingWriteObserver { count: 0 }
    }
}

impl WriteResponseObserver for CountingWriteObserver {
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
        self.count += 1;
    }
}
