use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::result::ReadRequestResult;
use crate::interface::telemetry::metrics::{
    CacheMetrics, RenderMetrics, ResponseMetrics,
};


pub struct HandleContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub request: &'c mut Apache2Request<'c>,
    pub cache_metrics: &'c dyn CacheMetrics,
    pub render_metrics: &'c dyn RenderMetrics,
    pub response_metrics: &'c dyn ResponseMetrics,
}

pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &SlippyRequest,
    ) -> HandleRequestResult;
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
        obj: &dyn RequestHandler,
        context: &HandleContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
    ) -> ();
}


#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::schema::handler::result::HandleOutcome;
    use crate::schema::slippy::request;
    use chrono::Utc;


    pub struct MockRequestHandler { }

    impl RequestHandler for MockRequestHandler {
        fn handle(
            &mut self,
            _context: &HandleContext,
            _request: &request::SlippyRequest,
        ) -> HandleRequestResult {
            return HandleRequestResult {
                before_timestamp: Utc::now(),
                after_timestamp: Utc::now(),
                result: Ok(HandleOutcome::NotHandled),
            };
        }
    }
}
