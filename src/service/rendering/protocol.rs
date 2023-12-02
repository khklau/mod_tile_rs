use crate::binding::renderd_protocol::{protoCmd, protocol};
use crate::schema::apache2::config::RenderdConfig;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::renderd::request::{RenderRequestCommand, RenderRequestVersion};
use crate::service::rendering::status::data_import_completion_time;


pub fn create_request(
    config: &RenderdConfig,
    slippy: &SlippyRequest,
) -> protocol {
    let completion_time = data_import_completion_time(config, &slippy.header.layer);
    let result = protocol {
        ver: RenderRequestVersion::Three as std::os::raw::c_int,
        cmd: RenderRequestCommand::Render as protoCmd,
        x: 0 as std::os::raw::c_int,
        y: 0 as std::os::raw::c_int,
        z: 0 as std::os::raw::c_int,
        xmlname: [0; 41usize],
        mimetype: [0; 41usize],
        options: [0; 41usize],
    };
    result
}
