use crate::apache2::request::Apache2Request;
use crate::schema::apache2::config::ModuleConfig;
use crate::interface::telemetry::metrics::{
    CacheMetrics, RenderMetrics, ResponseMetrics,
};


pub struct HandleContext<'c> {
    pub module_config: &'c ModuleConfig,
    pub request_context: &'c mut Apache2Request<'c>,
    pub cache_metrics: &'c dyn CacheMetrics,
    pub render_metrics: &'c dyn RenderMetrics,
    pub response_metrics: &'c dyn ResponseMetrics,
}
