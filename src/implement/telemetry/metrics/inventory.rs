use crate::interface::slippy::WriteResponseObserver;
use crate::interface::telemetry::{ ResponseMetrics, TileHandlingMetrics, };
use crate::implement::telemetry::metrics::response::ResponseAnalysis;
use crate::implement::telemetry::metrics::tile_handling::TileHandlingAnalysis;


#[cfg(not(test))]
pub struct MetricsState {
    pub response_analysis: ResponseAnalysis,
    pub tile_handling_analysis: TileHandlingAnalysis,
}

#[cfg(not(test))]
impl MetricsState {
    pub fn new() -> MetricsState {
        MetricsState {
            response_analysis: ResponseAnalysis::new(),
            tile_handling_analysis: TileHandlingAnalysis::new(),
        }
    }

    pub fn response_metrics(&self) -> &dyn ResponseMetrics {
        &self.response_analysis
    }

    pub fn tile_handling_metrics(&self) -> &dyn TileHandlingMetrics {
        &self.tile_handling_analysis
    }

    pub fn write_observers(&mut self) -> [&mut dyn WriteResponseObserver; 2] {
        let (response_analysis, tile_handling_analysis) = (
            &mut self.response_analysis,
            &mut self.tile_handling_analysis,
        );
        [response_analysis, tile_handling_analysis]
    }
}


#[cfg(test)]
pub struct MetricsState {
    pub response_analysis: ResponseAnalysisVariant,
    pub tile_handling_analysis: TileHandlingAnalysisVariant,
}

#[cfg(test)]
impl MetricsState {
    pub fn new() -> MetricsState {
        MetricsState {
            response_analysis: ResponseAnalysisVariant::Real(
                ResponseAnalysis::new()
            ),
            tile_handling_analysis: TileHandlingAnalysisVariant::Real(
                TileHandlingAnalysis::new()
            ),
        }
    }

    pub fn new_mock(
        response_analysis: ResponseAnalysisVariant,
        tile_handling_analysis: TileHandlingAnalysisVariant,
    ) -> MetricsState {
        MetricsState {
            response_analysis,
            tile_handling_analysis,
        }
    }

    pub fn response_metrics(&self) -> &dyn ResponseMetrics {
        match &self.response_analysis {
            ResponseAnalysisVariant::Real(analysis) => &(*analysis),
            ResponseAnalysisVariant::MockNoOp(analysis) => &(*analysis),
        }
    }

    pub fn tile_handling_metrics(&self) -> &dyn TileHandlingMetrics {
        match &self.tile_handling_analysis {
            TileHandlingAnalysisVariant::Real(analysis) => &(*analysis),
            TileHandlingAnalysisVariant::MockNoOp(analysis) => &(*analysis),
        }
    }

    pub fn write_observers(&mut self) -> [&mut dyn WriteResponseObserver; 2] {
        let (response_analysis, tile_handling_analysis) = (
            &mut self.response_analysis,
            &mut self.tile_handling_analysis,
        );
        [
            match response_analysis {
                ResponseAnalysisVariant::Real(analysis) => &mut *analysis,
                ResponseAnalysisVariant::MockNoOp(analysis) => &mut *analysis,
            },
            match tile_handling_analysis {
                TileHandlingAnalysisVariant::Real(analysis) => &mut *analysis,
                TileHandlingAnalysisVariant::MockNoOp(analysis) => &mut *analysis,
            }
        ]
    }
}

#[cfg(test)]
pub enum ResponseAnalysisVariant {
    Real(ResponseAnalysis),
    MockNoOp(crate::implement::telemetry::metrics::response::test_utils::MockNoOpResponseAnalysis),
}

#[cfg(test)]
pub enum TileHandlingAnalysisVariant {
    Real(TileHandlingAnalysis),
    MockNoOp(crate::implement::telemetry::metrics::tile_handling::test_utils::MockNoOpTileHandlingAnalysis),
}

pub struct MetricsInventory<'i> {
    pub response_metrics: &'i dyn ResponseMetrics,
    pub tile_handling_metrics: &'i dyn TileHandlingMetrics,
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
            metrics_state.response_metrics()
        };
        let tile_handling_metrics = if let Some(obj) = self.tile_handling_metrics {
            obj
        } else {
            metrics_state.tile_handling_metrics()
        };
        let metrics_inventory = MetricsInventory {
            response_metrics,
            tile_handling_metrics,
        };
        func(&metrics_inventory)
    }
}
