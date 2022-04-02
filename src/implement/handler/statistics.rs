use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::interface::handler::{ HandleContext, RequestHandler, };

use chrono::Utc;
use mime;

use std::collections::HashMap;
use std::vec::Vec;


pub struct StatisticsHandler { }

impl RequestHandler for StatisticsHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &request::SlippyRequest,
    ) -> HandleRequestResult {
        let before_timestamp = Utc::now();
        match request.body {
            request::BodyVariant::ReportStatistics => (),
            _ => {
                return HandleRequestResult {
                    before_timestamp,
                    after_timestamp: Utc::now(),
                    result: Ok(HandleOutcome::NotHandled),
                }
            },
        };
        let statistics = report(context);
        let response = response::SlippyResponse {
            header: response::Header::new(
                context.request.record,
                &mime::TEXT_PLAIN,
            ),
            body: response::BodyVariant::Statistics(statistics),
        };
        let after_timestamp = Utc::now();
        return HandleRequestResult {
            before_timestamp,
            after_timestamp,
            result: Ok(HandleOutcome::Handled(response)),
        };
    }
}

fn report(context: &HandleContext) -> response::Statistics {
    response::Statistics {
        number_response_200: 0,
        number_response_304: 0,
        number_response_404: 0,
        number_response_503: 0,
        number_response_5xx: 0,
        number_response_other: 0,
        number_fresh_cache: 0,
        number_old_cache: 0,
        number_very_old_cache: 0,
        number_fresh_render: 0,
        number_old_render: 0,
        number_very_old_render: 0,
        number_response_zoom: Vec::new(),
        number_tile_buffer_reads: 0,
        duration_tile_buffer_reads: 0,
        number_tile_buffer_read_zoom: Vec::new(),
        duration_tile_buffer_read_zoom: Vec::new(),
        number_response_200_by_layer: HashMap::new(),
        number_response_400_by_layer:  HashMap::new(),
    }
}
