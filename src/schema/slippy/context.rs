use crate::apache2::request::RequestContext;
use crate::schema::apache2::config::ModuleConfig;


pub struct ReadContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub request_context: &'c RequestContext<'c>,
}
