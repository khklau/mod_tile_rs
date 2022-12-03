use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::handler::HandleRequestObserver;
use crate::interface::slippy::{ReadRequestObserver, WriteResponseObserver,};
use crate::interface::telemetry::{
    ResponseMetrics, TelemetryInventory, TileHandlingMetrics,
};
use crate::implement::telemetry::counters::{
    HandleCounter, ReadCounter, WriteCounter,
};
use crate::implement::telemetry::response::ResponseAnalysis;
use crate::implement::telemetry::tile_handling::TileHandlingAnalysis;
use crate::implement::telemetry::transaction::TransactionTrace;

use std::result::Result;


pub struct TelemetryState {
    response_analysis: ResponseAnalysis,
    tile_handling_analysis: TileHandlingAnalysis,
    trans_trace: TransactionTrace,
    read_counter: ReadCounter,
    handle_counter: HandleCounter,
    write_counter: WriteCounter,
}

impl TelemetryState {
    pub fn new(config: &ModuleConfig) -> Result<TelemetryState, InvalidConfigError> {
        Ok(
            TelemetryState {
                response_analysis: ResponseAnalysis::new(config)?,
                tile_handling_analysis: TileHandlingAnalysis::new(config)?,
                trans_trace: TransactionTrace::new(config)?,
                read_counter: ReadCounter::new(config)?,
                handle_counter: HandleCounter::new(config)?,
                write_counter: WriteCounter::new(config)?,
            }
        )
    }
}

impl TelemetryInventory for TelemetryState {
    fn response_metrics(&self) -> &dyn ResponseMetrics {
        &self.response_analysis
    }

    fn tile_handling_metrics(&self) -> &dyn TileHandlingMetrics {
        &self.tile_handling_analysis
    }

    fn read_request_observers(&mut self) -> [&mut dyn ReadRequestObserver; 2] {
        [&mut self.trans_trace, &mut self.read_counter]
    }

    fn handle_request_observers(&mut self) -> [&mut dyn HandleRequestObserver; 2] {
        [&mut self.trans_trace, &mut self.handle_counter]
    }

    fn write_response_observers(&mut self) -> [&mut dyn WriteResponseObserver; 4] {
        [
            &mut self.trans_trace,
            &mut self.response_analysis,
            &mut self.tile_handling_analysis,
            &mut self.write_counter,
        ]
    }
}


#[cfg(test)]
impl TelemetryState {
    pub fn read_counter(&self) -> &ReadCounter {
        &&self.read_counter
    }

    pub fn handle_counter(&self) -> &HandleCounter {
        &self.handle_counter
    }

    pub fn write_counter(&self) -> &WriteCounter {
        &self.write_counter
    }
}
