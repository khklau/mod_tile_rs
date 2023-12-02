use crate::binding::apache2::request_rec;
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::core::memory::PoolStored;
use crate::interface::io::communication::CommunicationInventory;
use crate::interface::io::storage::StorageInventory;
use crate::interface::service::telemetry::TelemetryInventory;


pub struct HostContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
}

pub struct RequestContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub request: &'c Apache2Request<'c>,
}

impl<'c> RequestContext<'c> {
    pub fn new(
        record: &'c mut request_rec,
        module_config: &'c ModuleConfig,
    ) -> RequestContext<'c> {
        RequestContext {
            module_config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            request: Apache2Request::find_or_allocate_new(record).unwrap(),
        }
    }
}

pub struct IOContext<'c> {
    pub communication: &'c mut dyn CommunicationInventory,
    pub storage: &'c mut dyn StorageInventory,
}

pub struct ServicesContext<'c> {
    pub telemetry: &'c dyn TelemetryInventory,
}
