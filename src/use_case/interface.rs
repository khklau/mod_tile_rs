use crate::schema::handler::error::HandleError;
use crate::schema::slippy::response::SlippyResponse;
use crate::schema::slippy::request::{Header, ServeTileRequest,};


pub trait DescriptionUseCaseObserver {
    fn on_describe_layer(
        &mut self,
        request: &Header,
        handle_result: &Result<SlippyResponse, HandleError>,
        handler_name: &'static str,
    ) -> ();
}

pub trait StatisticsUseCaseObserver {
    fn on_report_statistics(
        &mut self,
        header: &Header,
        handle_result: &Result<SlippyResponse, HandleError>,
        handler_name: &'static str,
    ) -> ();
}

pub trait TileUseCaseObserver {
    fn on_fetch_tile(
        &mut self,
        header: &Header,
        body: &ServeTileRequest,
        handle_result: &Result<SlippyResponse, HandleError>,
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

    impl DescriptionUseCaseObserver for NoOpHandleRequestObserver {
        fn on_describe_layer(
            &mut self,
            _header: &Header,
            _handle_result: &Result<SlippyResponse, HandleError>,
            _handler_name: &'static str,
        ) -> () {
        }
    }

    impl StatisticsUseCaseObserver for NoOpHandleRequestObserver {
        fn on_report_statistics(
            &mut self,
            _header: &Header,
            _handle_result: &Result<SlippyResponse, HandleError>,
            _handler_name: &'static str,
        ) -> () {
        }
    }

    impl TileUseCaseObserver for NoOpHandleRequestObserver {
        fn on_fetch_tile(
            &mut self,
            _header: &Header,
            _body: &ServeTileRequest,
            _handle_result: &Result<SlippyResponse, HandleError>,
            _handler_name: &'static str,
        ) -> () {
        }
    }
}
