use crate::service::telemetry::interface::TelemetryInventory;
use crate::service::rendering::interface::RenderingInventory;


pub struct ServicesContext<'c> {
    pub telemetry: &'c dyn TelemetryInventory,
    pub rendering: &'c mut dyn RenderingInventory,
}
