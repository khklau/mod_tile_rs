use crate::schema::apache2::request::Apache2Request;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::io::communication::HttpResponseWriter;
use crate::interface::context::{HostContext, RequestContext,};


pub type ReadRequestFunc = fn(&HostContext, &Apache2Request) -> ReadOutcome;

pub type WriteResponseFunc = fn(&RequestContext, &SlippyResponse, &mut dyn HttpResponseWriter) -> WriteOutcome;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        context: &HostContext,
        request: &Apache2Request,
        read_outcome: &ReadOutcome,
        read_func_name: &'static str,
    ) -> ();
}

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        context: &RequestContext,
        response: &SlippyResponse,
        writer: &dyn HttpResponseWriter,
        write_outcome: &WriteOutcome,
        write_func_name: &'static str,
        read_outcome: &ReadOutcome,
        handle_outcome: &HandleOutcome,
    ) -> ();
}


#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub struct NoOpReadRequestObserver { }

    impl NoOpReadRequestObserver {
        pub fn new() -> NoOpReadRequestObserver {
            NoOpReadRequestObserver { }
        }
    }

    impl ReadRequestObserver for NoOpReadRequestObserver {
        fn on_read(
            &mut self,
            _context: &HostContext,
            _request: &Apache2Request,
            _read_outcome: &ReadOutcome,
            _read_func_name: &'static str,
        ) -> () {
        }
    }

    pub struct NoOpWriteResponseObserver { }

    impl NoOpWriteResponseObserver {
        pub fn new() -> NoOpWriteResponseObserver {
            NoOpWriteResponseObserver { }
        }
    }

    impl WriteResponseObserver for NoOpWriteResponseObserver {
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
        }
    }
}