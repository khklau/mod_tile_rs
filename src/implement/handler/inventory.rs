use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::telemetry::TelemetryInventory;
use crate::interface::handler::{
    HandlerInventory, HandleRequestObserver, RequestHandler
};
use crate::implement::handler::description::DescriptionHandlerState;
use crate::implement::handler::statistics::StatisticsHandlerState;
use crate::implement::handler::tile::TileHandlerState;


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

impl HandlerInventory for HandlerState {
    fn request_handlers(&mut self) -> [&mut dyn RequestHandler; 3] {
        [
            &mut self.description,
            &mut self.statistics,
            &mut self.tile,
        ]
    }
}

pub struct HandlerObserverInventory { }

impl HandlerObserverInventory {
    pub fn handle_observers<'i>(
        telemetry: &'i mut dyn TelemetryInventory
    ) -> [&'i mut dyn HandleRequestObserver; 2] {
        let [read_observer_0, read_observer_1] = telemetry.handle_request_observers();
        return [read_observer_0, read_observer_1];
    }
}
