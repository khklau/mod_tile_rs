use crate::binding::apache2::request_rec;
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::result::ReadOutcome;
use crate::interface::apache2::PoolStored;


pub struct HandleContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub request: &'c mut Apache2Request<'c>,
}

impl<'c> HandleContext<'c> {
    pub fn new(
        record: &'c mut request_rec,
        module_config: &'c ModuleConfig,
    ) -> HandleContext<'c> {
        HandleContext {
            module_config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
            request: Apache2Request::find_or_allocate_new(record).unwrap(),
        }
    }
}

pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &SlippyRequest,
    ) -> HandleOutcome;

    fn type_name(&self) -> &'static str;
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
        context: &HandleContext,
        request: &SlippyRequest,
        handle_outcome: &HandleOutcome,
        handler_name: &'static str,
        read_outcome: &ReadOutcome,
    ) -> ();
}


#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::schema::slippy::request;


    pub struct MockRequestHandler { }

    impl RequestHandler for MockRequestHandler {
        fn handle(
            &mut self,
            _context: &HandleContext,
            _request: &request::SlippyRequest,
        ) -> HandleOutcome {
            HandleOutcome::Ignored
        }

        fn type_name(&self) -> &'static str {
            std::any::type_name::<Self>()
        }
    }
}
