use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::source::TileSource;
use crate::interface::handler::{ HandleContext, RequestHandler, };
use crate::interface::telemetry::MetricsInventory;

use chrono::Utc;
use http::status::StatusCode;
use mime;


pub struct StatisticsHandler<'h> {
    metrics: &'h MetricsInventory<'h>,
}

impl<'h> StatisticsHandler<'h> {
    pub fn new(metrics: &'h MetricsInventory) -> StatisticsHandler<'h> {
        StatisticsHandler {
            metrics,
        }
    }

    fn report(
        &self,
        _context: &HandleContext
    ) -> response::Statistics {
        let mut result = response::Statistics::new();
        for status_code in self.metrics.response_metrics.iterate_status_codes_responded() {
            let count = self.metrics.response_metrics.count_response_by_status_code(status_code);
            match status_code {
                &StatusCode::OK => { result.number_response_200 = count; },
                &StatusCode::NOT_MODIFIED => { result.number_response_304 = count; },
                &StatusCode::NOT_FOUND => { result.number_response_404 = count; },
                &StatusCode::SERVICE_UNAVAILABLE => { result.number_response_503 = count; },
                &StatusCode::INTERNAL_SERVER_ERROR => { result.number_response_5xx = count; },
                _ => { result.number_response_other += count; }
            }
        }
        result.number_fresh_cache = self.metrics.tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Fresh,
        );
        result.number_old_cache = self.metrics.tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::Old,
        );
        result.number_very_old_cache = self.metrics.tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Cache,
            &TileAge::VeryOld,
        );
        result.number_fresh_render = self.metrics.tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Fresh,
        );
        result.number_old_render = self.metrics.tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::Old,
        );
        result.number_very_old_render = self.metrics.tile_handling_metrics.count_handled_tile_by_source_and_age(
            &TileSource::Render,
            &TileAge::VeryOld,
        );
        for zoom_level in self.metrics.response_metrics.iterate_valid_zoom_levels() {
            let any_count = self.metrics.response_metrics.count_response_by_zoom_level(zoom_level);
            result.number_successful_response_by_zoom[zoom_level as usize] = any_count;
            let tile_count = self.metrics.response_metrics.count_tile_response_by_zoom_level(zoom_level);
            result.number_tile_response_by_zoom[zoom_level as usize] = tile_count;
            let tile_duration = self.metrics.response_metrics.tally_tile_response_duration_by_zoom_level(zoom_level);
            result.duration_tile_response_by_zoom[zoom_level as usize] = tile_duration;
        }
        result.total_number_tile_response = self.metrics.response_metrics.count_total_tile_response();
        result.total_duration_tile_response = self.metrics.response_metrics.tally_total_tile_response_duration();
        for layer in self.metrics.response_metrics.iterate_layers_responded() {
            let count_200 = self.metrics.response_metrics.count_response_by_layer_and_status_code(layer, &http::StatusCode::OK);
            result.number_response_200_by_layer.insert(layer.clone(), count_200);
            let count_404 = self.metrics.response_metrics.count_response_by_layer_and_status_code(layer, &http::StatusCode::NOT_FOUND);
            result.number_response_404_by_layer.insert(layer.clone(), count_404);
        }
        return result;
    }
}

impl<'h> RequestHandler for StatisticsHandler<'h> {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &request::SlippyRequest,
    ) -> HandleOutcome {
        let before_timestamp = Utc::now();
        match request.body {
            request::BodyVariant::ReportStatistics => (),
            _ => {
                return HandleOutcome::Ignored;
            },
        };
        let statistics = self.report(context);
        let response = response::SlippyResponse {
            header: response::Header::new(
                context.request.record,
                &mime::TEXT_PLAIN,
            ),
            body: response::BodyVariant::Statistics(statistics),
        };
        let after_timestamp = Utc::now();
        return HandleOutcome::Processed(
            HandleRequestResult {
                before_timestamp,
                after_timestamp,
                result: Ok(response),
            }
        );
    }
}
