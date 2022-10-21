use crate::schema::apache2::config::ModuleConfig;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler, };
use crate::interface::telemetry::MetricsInventory;
use crate::implement::handler::description::{ DescriptionHandler, DescriptionHandlerState, };
use crate::implement::handler::statistics::{ StatisticsHandler, StatisticsHandlerState, };
use crate::implement::handler::tile::{ TileHandler, TileHandlerState, };
use crate::implement::telemetry::tracing::inventory::TracingState;


pub struct HandlerInventory<'i> {
    pub handlers: [&'i mut dyn RequestHandler; 3],
    pub handle_observers: [&'i mut dyn HandleRequestObserver; 1],
}

pub struct HandlerFactory<'f> {
    pub handlers: Option<[&'f mut dyn RequestHandler; 3]>,
    pub handle_observers: Option<[&'f mut dyn HandleRequestObserver; 1]>,
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
        tracing_state: &mut TracingState,
        description_handler_state: &mut DescriptionHandlerState,
        statistics_handler_state: &mut StatisticsHandlerState,
        tile_handler_state: &mut TileHandlerState,
        metrics_inventory: &MetricsInventory,
        func: F,
    ) -> R
    where
        F: FnOnce(&mut HandlerInventory) -> R {
        let mut description_handler = DescriptionHandler::new(description_handler_state);
        let mut statistics_handler = StatisticsHandler::new(statistics_handler_state, &metrics_inventory);
        let mut tile_handler = TileHandler::new(tile_handler_state, None);
        let mut handler_inventory = HandlerInventory {
            handlers: match &mut self.handlers {
                // TODO: find a nicer way to copy, clone method doesn't work with trait object elements
                Some([handler_0, handler_1, handler_2]) => [*handler_0, *handler_1, *handler_2],
                None => [&mut description_handler, &mut statistics_handler, &mut tile_handler],
            },
            handle_observers: match &mut self.handle_observers {
                // TODO: find a nicer way to copy, clone method doesn't work with trait object elements
                Some([observer_0]) => [*observer_0],
                None => [&mut tracing_state.trans_trace],
            }
        };
        func(&mut handler_inventory)
    }
}
