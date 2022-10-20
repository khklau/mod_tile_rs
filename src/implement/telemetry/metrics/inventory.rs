use crate::interface::telemetry::{
    MetricsInventory,
    ResponseMetrics,
    TileHandlingMetrics,
};
use crate::implement::telemetry::metrics::response::ResponseAnalysis;
use crate::implement::telemetry::metrics::tile_handling::TileHandlingAnalysis;


pub struct MetricsState {
    pub response_analysis: ResponseAnalysis,
    pub tile_handling_analysis: TileHandlingAnalysis,
}

impl MetricsState {
    pub fn new() -> MetricsState {
        MetricsState {
            response_analysis: ResponseAnalysis::new(),
            tile_handling_analysis: TileHandlingAnalysis::new(),
        }
    }
}


pub struct MetricsFactory<'f> {
    response_metrics: Option<&'f dyn ResponseMetrics>,
    tile_handling_metrics: Option<&'f dyn TileHandlingMetrics>,
}

impl<'f> MetricsFactory<'f> {
    pub fn new() -> MetricsFactory<'f> {
        MetricsFactory {
            response_metrics: None,
            tile_handling_metrics: None,
        }
    }

    pub fn with_metrics_inventory<F, R>(
        &self,
        metrics_state: &MetricsState,
        func: F,
    ) -> R
    where
        F: FnOnce(&MetricsInventory) -> R {
        let response_metrics = if let Some(obj) = self.response_metrics {
            obj
        } else {
            &metrics_state.response_analysis
        };
        let tile_handling_metrics = if let Some(obj) = self.tile_handling_metrics {
            obj
        } else {
            &metrics_state.tile_handling_analysis
        };
        let metrics_inventory = MetricsInventory {
            response_metrics,
            tile_handling_metrics,
        };
        func(&metrics_inventory)
    }
}
