use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::apache2::HttpResponseWriter;


pub struct ReadContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
}

pub struct WriteContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub request: &'c Apache2Request<'c>,
}

pub type ReadRequestFunc = fn(&ReadContext, &Apache2Request) -> ReadOutcome;

pub type WriteResponseFunc = fn(&WriteContext, &SlippyResponse, &mut dyn HttpResponseWriter) -> WriteOutcome;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        context: &ReadContext,
        request: &Apache2Request,
        read_outcome: &ReadOutcome,
        read_func_name: &'static str,
    ) -> ();
}

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        context: &WriteContext,
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
            _context: &ReadContext,
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
