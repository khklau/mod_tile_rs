use crate::schema::apache2::config::ModuleConfig;
use crate::apache2::request::Apache2Request;
use crate::apache2::response::Apache2Response;


pub struct ReadContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub request: &'c Apache2Request<'c>,
}

pub struct WriteContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub response_context: &'c mut Apache2Response<'c>,
}
