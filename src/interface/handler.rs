use crate::binding::apache2::request_rec;
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::result::ReadOutcome;
use crate::interface::apache2::PoolStored;
use crate::interface::communication::CommunicationInventory;
use crate::interface::storage::StorageInventory;
use crate::interface::telemetry::TelemetryInventory;


pub struct HandleIOContext<'c> {
    pub communication: &'c mut dyn CommunicationInventory,
    pub storage: &'c mut dyn StorageInventory,
}

impl<'c> HandleIOContext<'c> {
    pub fn new(
        communication: &'c mut dyn CommunicationInventory,
        storage: &'c mut dyn StorageInventory,
    ) -> HandleIOContext<'c> {
        HandleIOContext {
            communication,
            storage,
        }
    }
}

pub struct HandleContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub request: &'c mut Apache2Request<'c>,
    pub telemetry: &'c dyn TelemetryInventory,
}

impl<'c> HandleContext<'c> {
    pub fn new(
        record: &'c mut request_rec,
        module_config: &'c ModuleConfig,
        telemetry: &'c dyn TelemetryInventory,
    ) -> HandleContext<'c> {
        HandleContext {
            module_config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
            request: Apache2Request::find_or_allocate_new(record).unwrap(),
            telemetry,
        }
    }
}

pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        io: &mut HandleIOContext,
        request: &SlippyRequest,
    ) -> HandleOutcome;

    fn type_name(&self) -> &'static str;
}

pub trait HandlerInventory {
    fn request_handlers(&mut self) -> [&mut dyn RequestHandler; 3];
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
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


    pub struct NoOpRequestHandler { }

    impl RequestHandler for NoOpRequestHandler {
        fn handle(
            &mut self,
            _context: &HandleContext,
            _io: &mut HandleIOContext,
            _request: &request::SlippyRequest,
        ) -> HandleOutcome {
            HandleOutcome::Ignored
        }

        fn type_name(&self) -> &'static str {
            std::any::type_name::<Self>()
        }
    }

    pub struct NoOpHandleRequestObserver {}

    impl NoOpHandleRequestObserver {
        pub fn new() -> NoOpHandleRequestObserver {
            NoOpHandleRequestObserver { }
        }
    }

    impl HandleRequestObserver for NoOpHandleRequestObserver {
        fn on_handle(
            &mut self,
            _request: &SlippyRequest,
            _handle_outcome: &HandleOutcome,
            _handler_name: &'static str,
            _read_outcome: &ReadOutcome,
        ) -> () {
        }
    }
}
