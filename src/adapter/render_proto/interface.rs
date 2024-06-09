use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::renderd::request::RenderRequest;
use crate::schema::renderd::error::InvalidParameterError;
use crate::schema::slippy::request::Header;
use crate::framework::apache2::context::HostContext;

use std::result::Result;


pub struct RenderProtoContext<'c> {
    pub host_context: HostContext<'c>,
}

impl<'c> RenderProtoContext<'c> {
    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host_context.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host_context.host
    }
}

pub trait ToRenderProto {
    fn to_render_proto(
        &self,
        context: &RenderProtoContext,
        header: &Header,
    ) -> Result<RenderRequest, InvalidParameterError>;
}
