use crate::schema::apache2::config::ModuleConfig;
use crate::apache2::connection::Connection;
use crate::apache2::virtual_host::VirtualHost;
use crate::apache2::request::Apache2Request;
use crate::apache2::response::Apache2Response;


pub struct ReadContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub request: &'c Apache2Request<'c>,
}

pub struct WriteContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub response: &'c mut Apache2Response<'c>,
}
