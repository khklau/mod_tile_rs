use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::service::telemetry::interface::TelemetryInventory;
use crate::use_case::interface::{
    DescriptionUseCaseObserver,
    HandleRequestObserver,
    StatisticsUseCaseObserver,
    TileUseCaseObserver,
};
use crate::use_case::description::DescriptionHandlerState;
use crate::use_case::statistics::StatisticsHandlerState;
use crate::use_case::tile::TileHandlerState;


pub struct HandlerState {
    pub description: DescriptionHandlerState,
    pub statistics: StatisticsHandlerState,
    pub tile: TileHandlerState,
}

impl HandlerState {
    pub fn new(config: &ModuleConfig) -> Result<HandlerState, InvalidConfigError> {
        Ok(
            HandlerState {
                description: DescriptionHandlerState::new(config)?,
                statistics: StatisticsHandlerState::new(config)?,
                tile: TileHandlerState::new(config)?,
            }
        )
    }
}

pub struct HandlerObserverInventory { }

impl HandlerObserverInventory {
    pub fn description_use_case_observers<'i>(
        telemetry: &'i mut dyn TelemetryInventory
    ) -> [&'i mut dyn DescriptionUseCaseObserver; 2] {
        let [read_observer_0, read_observer_1] = telemetry.description_use_case_observers();
        return [read_observer_0, read_observer_1];
    }

    pub fn statistics_use_case_observers<'i>(
        telemetry: &'i mut dyn TelemetryInventory
    ) -> [&'i mut dyn StatisticsUseCaseObserver; 2] {
        let [read_observer_0, read_observer_1] = telemetry.statistics_use_case_observers();
        return [read_observer_0, read_observer_1];
    }

    pub fn tile_use_case_observers<'i>(
        telemetry: &'i mut dyn TelemetryInventory
    ) -> [&'i mut dyn TileUseCaseObserver; 2] {
        let [read_observer_0, read_observer_1] = telemetry.tile_use_case_observers();
        return [read_observer_0, read_observer_1];
    }
}
