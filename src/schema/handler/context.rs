use crate::apache2::connection::Connection;
use crate::apache2::request::Apache2Request;
use crate::apache2::virtual_host::VirtualHost;
use crate::schema::apache2::config::ModuleConfig;
use crate::interface::telemetry::metrics::{
    CacheMetrics, RenderMetrics, ResponseMetrics,
};


pub struct HandleContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub host: &'c VirtualHost<'c>,
    pub connection: &'c Connection<'c>,
    pub request: &'c mut Apache2Request<'c>,
    pub cache_metrics: &'c dyn CacheMetrics,
    pub render_metrics: &'c dyn RenderMetrics,
    pub response_metrics: &'c dyn ResponseMetrics,
}
