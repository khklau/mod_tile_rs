use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::telemetry::TelemetryInventory;
use crate::interface::handler::{
    HandlerInventory2, HandleRequestObserver, RequestHandler, RequestHandler2
};
use crate::implement::handler::description::{ DescriptionHandler, DescriptionHandlerState, };
use crate::implement::handler::statistics::{ StatisticsHandler, StatisticsHandlerState, };
use crate::implement::handler::tile::{ TileHandler, TileHandlerState, };
use crate::implement::telemetry::inventory::TelemetryState;


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

impl HandlerInventory2 for HandlerState {
    fn request_handlers(&mut self) -> [&mut dyn RequestHandler2; 3] {
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
