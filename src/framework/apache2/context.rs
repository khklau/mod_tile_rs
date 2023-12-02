use crate::binding::apache2::request_rec;
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::core::memory::PoolStored;
use crate::io::communication::interface::CommunicationInventory;
use crate::io::storage::interface::StorageInventory;
use crate::service::telemetry::interface::TelemetryInventory;


pub struct HostContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
}

pub struct RequestContext<'c> {
    pub host_context: HostContext<'c>,
    pub request: &'c Apache2Request<'c>,
}

impl<'c> RequestContext<'c> {
    pub fn new(
        record: &'c mut request_rec,
        module_config: &'c ModuleConfig,
    ) -> RequestContext<'c> {
        RequestContext {
            host_context: HostContext {
                module_config,
                host: VirtualHost::find_or_allocate_new(record).unwrap(),
            },
            request: Apache2Request::find_or_allocate_new(record).unwrap(),
        }
    }

    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host_context.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host_context.host
    }
}