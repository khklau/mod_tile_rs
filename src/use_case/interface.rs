use crate::schema::handler::result::HandleOutcome;
use crate::schema::slippy::request::SlippyRequest;


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
