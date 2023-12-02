use crate::service::telemetry::interface::TelemetryInventory;


pub struct ServicesContext<'c> {
    pub telemetry: &'c dyn TelemetryInventory,
}
