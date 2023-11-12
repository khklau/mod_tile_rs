use crate::binding::renderd_protocol::{protocol, protoCmd};
use crate::schema::apache2::config::RenderdConfig;
use crate::schema::renderd::error::RenderError;
use crate::schema::renderd::request::{RenderRequestCommand, RenderRequestVersion};
use crate::schema::renderd::status::DataImportStatus;
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::tile::identity::{LayerName, TileIdentity,};
use crate::interface::io::communication::CommunicationInventory;


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

pub trait TileRenderer {
    fn render_tile(
        communication: &mut dyn CommunicationInventory,
        tile_id: TileIdentity,
        request: &protocol,
        response: &mut protocol,
    ) -> Result<(), RenderError>;

    fn get_data_import_status(
        communication: &mut dyn CommunicationInventory,
        config: &RenderdConfig,
        layer_name: &LayerName,
    ) -> DataImportStatus;
}
