use crate::binding::apache2::request_rec;
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::core::memory::PoolStored;


pub struct HostContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
}

impl<'c> HostContext<'c> {
    pub fn new(
        config: &'c ModuleConfig,
        request: &'c request_rec,
    ) -> HostContext<'c> {
        HostContext {
            module_config: &config,
            host: VirtualHost::find_or_allocate_new(request).unwrap(),
        }
    }
}
