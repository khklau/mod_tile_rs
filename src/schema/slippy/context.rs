use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::schema::apache2::config::ModuleConfig;


pub struct ReadContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub request_context: &'c RequestContext<'c>,
}

pub struct WriteContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub response_context: &'c mut ResponseContext<'c>,
}
