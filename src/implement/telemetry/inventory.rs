use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::handler::HandleRequestObserver;
use crate::interface::slippy::{ReadRequestObserver, WriteResponseObserver,};
use crate::interface::telemetry::{
    ResponseMetrics, TelemetryInventory, TileHandlingMetrics,
};
use crate::implement::telemetry::response::ResponseAnalysis;
use crate::implement::telemetry::tile_handling::TileHandlingAnalysis;
use crate::implement::telemetry::transaction::TransactionTrace;

use std::option::Option;
use std::result::Result;


pub struct TelemetryState {
    response_analysis: ResponseAnalysis,
    tile_handling_analysis: TileHandlingAnalysis,
    trans_trace: TransactionTrace,
}

impl TelemetryState {
    pub fn new(config: &ModuleConfig) -> Result<TelemetryState, InvalidConfigError> {
        Ok(
            TelemetryState {
                response_analysis: ResponseAnalysis::new(config)?,
                tile_handling_analysis: TileHandlingAnalysis::new(config)?,
                trans_trace: TransactionTrace { },
            }
        )
    }

    pub fn read_request_observers(&mut self) -> [&mut dyn ReadRequestObserver; 1] {
        [&mut self.trans_trace]
    }

    pub fn handle_request_observers(&mut self) -> [&mut dyn HandleRequestObserver; 1] {
        [&mut self.trans_trace]
    }

    pub fn write_response_observers(&mut self) -> [&mut dyn WriteResponseObserver; 3] {
        [
            &mut self.trans_trace,
            &mut self.response_analysis,
            &mut self.tile_handling_analysis,
        ]
    }
}

impl TelemetryInventory for TelemetryState {
    fn response_metrics(&self) -> &dyn ResponseMetrics {
        &self.response_analysis
    }

    fn tile_handling_metrics(&self) -> &dyn TileHandlingMetrics {
        &self.tile_handling_analysis
    }

    fn read_request_observers(&mut self) -> [&mut dyn ReadRequestObserver; 1] {
        [&mut self.trans_trace]
    }

    fn handle_request_observers(&mut self) -> [&mut dyn HandleRequestObserver; 1] {
        [&mut self.trans_trace]
    }

    fn write_response_observers(&mut self) -> [&mut dyn WriteResponseObserver; 3] {
        [
            &mut self.trans_trace,
            &mut self.response_analysis,
            &mut self.tile_handling_analysis,
        ]
    }
}
