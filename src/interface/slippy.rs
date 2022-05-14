use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::apache2::Writer;


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

pub type WriteResponseFunc = fn(&WriteContext, &SlippyResponse, &mut dyn Writer) -> WriteOutcome;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        func: ReadRequestFunc,
        context: &ReadContext,
        request: &Apache2Request,
        read_outcome: &ReadOutcome,
    ) -> ();
}

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        context: &WriteContext,
        response: &SlippyResponse,
        writer: &dyn Writer,
        write_outcome: &WriteOutcome,
        func: WriteResponseFunc,
        read_outcome: &ReadOutcome,
        handle_outcome: &HandleOutcome,
    ) -> ();
}
