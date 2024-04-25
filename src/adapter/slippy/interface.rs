use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::http::request::HttpRequest;
use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::error::{ReadError, WriteError,};
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::response::SlippyResponse;
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

pub type ReadRequestFunc = fn(&ReadContext, &HttpRequest) -> Result<SlippyRequest, ReadError>;

pub struct WriteContext<'c> {
    pub host_context: HostContext<'c>,
    pub request: &'c SlippyRequest,
}

impl<'c> WriteContext<'c> {
    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host_context.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host_context.host
    }
}

pub type WriteResponseFunc = fn(&WriteContext, &SlippyResponse, &mut dyn HttpResponseWriter) -> Result<HttpResponse, WriteError>;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        context: &ReadContext,
        request: &HttpRequest,
        read_outcome: &Result<SlippyRequest, ReadError>,
        read_func_name: &'static str,
    ) -> ();
}

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        context: &WriteContext,
        response: &SlippyResponse,
        writer: &dyn HttpResponseWriter,
        write_result: &Result<HttpResponse, WriteError>,
        write_func_name: &'static str,
        request: &SlippyRequest,
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
            _read_outcome: &Result<SlippyRequest, ReadError>,
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
            _write_result: &Result<HttpResponse, WriteError>,
            _write_func_name: &'static str,
            _request: &SlippyRequest,
        ) -> () {
        }
    }
}
