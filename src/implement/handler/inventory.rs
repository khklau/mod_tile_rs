use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::interface::telemetry::TelemetryInventory;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler, };
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

pub struct HandlerObserverInventory { }

impl HandlerObserverInventory {
    pub fn handle_observers<'i>(
        telemetry: &'i mut dyn TelemetryInventory
    ) -> [&'i mut dyn HandleRequestObserver; 2] {
        let [read_observer_0, read_observer_1] = telemetry.handle_request_observers();
        return [read_observer_0, read_observer_1];
    }
}


pub struct HandlerInventory<'i> {
    pub handlers: [&'i mut dyn RequestHandler; 3],
}

pub struct HandlerFactory<'f> {
    pub handlers: Option<[&'f mut dyn RequestHandler; 3]>,
    pub handle_observers: Option<[&'f mut dyn HandleRequestObserver; 2]>,
}

impl<'f> HandlerFactory<'f> {
    pub fn new() -> HandlerFactory<'f> {
        HandlerFactory {
            handlers: None,
            handle_observers: None,
        }
    }

    pub fn with_handler_inventory<F, R>(
        &mut self,
        _module_config: &ModuleConfig,
        telemetry_state: &TelemetryState,
        handler_state: &mut HandlerState,
        func: F,
    ) -> R
    where
        F: FnOnce(&mut HandlerInventory) -> R {
        let (descr_state, stats_state, tile_state) = (
            &mut handler_state.description,
            &mut handler_state.statistics,
            &mut handler_state.tile,
        );
        let mut description_handler = DescriptionHandler::new(descr_state);
        let mut statistics_handler = StatisticsHandler::new(stats_state, &(*telemetry_state));
        let mut tile_handler = TileHandler::new(tile_state, None);
        let mut handler_inventory = HandlerInventory {
            handlers: match &mut self.handlers {
                // TODO: find a nicer way to copy, clone method doesn't work with trait object elements
                Some([handler_0, handler_1, handler_2]) => [*handler_0, *handler_1, *handler_2],
                None => [&mut description_handler, &mut statistics_handler, &mut tile_handler],
            },
        };
        func(&mut handler_inventory)
    }
}
