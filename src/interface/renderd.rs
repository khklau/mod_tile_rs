use crate::binding::renderd_protocol::{protocol, protoCmd};
use crate::schema::apache2::config::RenderdConfig;
use crate::schema::renderd::request::{RenderRequestCommand, RenderRequestVersion};
use crate::schema::slippy::request::SlippyRequest;


pub fn create_request(
    _config: &RenderdConfig,
    _slippy: &SlippyRequest,
) -> protocol {
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
