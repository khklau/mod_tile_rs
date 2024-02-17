use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::http::request::HttpRequest;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::io::communication::interface::HttpResponseWriter;
use crate::framework::apache2::context::HostContext;


pub struct ReadContext<'c> {
    pub host_context: HostContext<'c>,
}

impl<'c> ReadContext<'c> {
    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host_context.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host_context.host
    }
}

pub type ReadRequestFunc = fn(&ReadContext, &HttpRequest) -> ReadOutcome;

pub struct WriteContext<'c> {
    pub host_context: HostContext<'c>,
    pub read_outcome: &'c ReadOutcome,
}

impl<'c> WriteContext<'c> {
    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host_context.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host_context.host
    }
}

pub type WriteResponseFunc = fn(&WriteContext, &SlippyResponse, &mut dyn HttpResponseWriter) -> WriteOutcome;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        context: &ReadContext,
        request: &HttpRequest,
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
            _request: &HttpRequest,
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
