use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::result::ReadOutcome;


pub struct HandleContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub request: &'c mut Apache2Request<'c>,
}

pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &SlippyRequest,
    ) -> HandleOutcome;
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
        context: &HandleContext,
        request: &SlippyRequest,
        handle_outcome: &HandleOutcome,
        obj: &dyn RequestHandler,
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
    }
}
