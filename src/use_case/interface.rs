use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;
use crate::io::interface::IOContext;
use crate::framework::apache2::context::RequestContext;
use crate::service::interface::ServicesContext;


pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &RequestContext,
        io: &mut IOContext,
        services: &mut ServicesContext,
        request: &SlippyRequest,
    ) -> HandleOutcome;

    fn type_name(&self) -> &'static str;
}

pub trait HandlerInventory {
    fn request_handlers(&mut self) -> [&mut dyn RequestHandler; 2];
}

pub trait HandleRequestObserver {
    fn on_handle(
        &mut self,
        request: &SlippyRequest,
        handle_outcome: &HandleOutcome,
        handler_name: &'static str,
    ) -> ();
}

pub trait DescriptionUseCaseObserver {
    fn on_describe_layer(
        &mut self,
        request: &SlippyRequest,
        handle_outcome: &HandleOutcome,
        handler_name: &'static str,
    ) -> ();
}

pub trait StatisticsUseCaseObserver {
    fn on_report_statistics(
        &mut self,
        request: &SlippyRequest,
        handle_outcome: &HandleOutcome,
        handler_name: &'static str,
    ) -> ();
}

pub trait TileUseCaseObserver {
    fn on_fetch_tile(
        &mut self,
        request: &SlippyRequest,
        handle_outcome: &HandleOutcome,
        handler_name: &'static str,
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
            _context: &RequestContext,
            _io: &mut IOContext,
            _services: &mut ServicesContext,
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
        ) -> () {
        }
    }

    impl DescriptionUseCaseObserver for NoOpHandleRequestObserver {
        fn on_describe_layer(
            &mut self,
            _request: &SlippyRequest,
            _handle_outcome: &HandleOutcome,
            _handler_name: &'static str,
        ) -> () {
        }
    }

    impl StatisticsUseCaseObserver for NoOpHandleRequestObserver {
        fn on_report_statistics(
            &mut self,
            _request: &SlippyRequest,
            _handle_outcome: &HandleOutcome,
            _handler_name: &'static str,
        ) -> () {
        }
    }

    impl TileUseCaseObserver for NoOpHandleRequestObserver {
        fn on_fetch_tile(
            &mut self,
            _request: &SlippyRequest,
            _handle_outcome: &HandleOutcome,
            _handler_name: &'static str,
        ) -> () {
        }
    }
}
