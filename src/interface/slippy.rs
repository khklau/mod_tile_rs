use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::result::{ ReadRequestResult, WriteResponseResult };
use crate::framework::apache2::response::ResponseWriter;


pub struct ReadContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
}

pub struct WriteContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub response: &'c mut ResponseWriter<'c>,
}

pub type ReadRequestFunc = fn(&ReadContext, &Apache2Request) -> ReadRequestResult;

pub type WriteResponseFunc = fn(&mut WriteContext, &SlippyResponse) -> WriteResponseResult;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        func: ReadRequestFunc,
        context: &ReadContext,
        result: &ReadRequestResult,
    ) -> ();
}

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        func: WriteResponseFunc,
        context: &WriteContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
        write_result: &WriteResponseResult,
    ) -> ();
}
