use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::adapter::slippy::interface::{ReadRequestObserver, WriteResponseObserver,};
use crate::service::telemetry::interface::{
    ResponseMetrics, TelemetryInventory, TileHandlingMetrics,
};
use crate::service::telemetry::counters::{
    HandleCounter, ReadCounter, WriteCounter,
};
use crate::service::telemetry::response::ResponseAnalysis;
use crate::service::telemetry::tile_handling::TileHandlingAnalysis;
use crate::service::telemetry::transaction::TransactionTrace;
use crate::use_case::interface::{
    DescriptionUseCaseObserver,
    StatisticsUseCaseObserver,
    TileUseCaseObserver,
};

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

    fn description_use_case_observers(&mut self) -> [&mut dyn DescriptionUseCaseObserver; 2] {
        [&mut self.trans_trace, &mut self.handle_counter]
    }

    fn statistics_use_case_observers(&mut self) -> [&mut dyn StatisticsUseCaseObserver; 2] {
        [&mut self.trans_trace, &mut self.handle_counter]
    }

    fn tile_use_case_observers(&mut self) -> [&mut dyn TileUseCaseObserver; 2] {
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
