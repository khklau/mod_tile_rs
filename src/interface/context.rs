use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::interface::communication::CommunicationInventory;
use crate::interface::storage::StorageInventory;
use crate::interface::telemetry::TelemetryInventory;


pub struct HostContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
}

pub struct RequestContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub request: &'c Apache2Request<'c>,
}

pub struct IOContext<'c> {
    pub communication: &'c mut dyn CommunicationInventory,
    pub storage: &'c mut dyn StorageInventory,
}

pub struct ServicesContext<'c> {
    pub telemetry: &'c dyn TelemetryInventory,
}
