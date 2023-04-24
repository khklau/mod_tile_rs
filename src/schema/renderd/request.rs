use crate::binding::renderd_protocol::{
    protoCmd,
    protoCmd_cmdDirty,
    protoCmd_cmdRender,
    protoCmd_cmdRenderBulk,
    protoCmd_cmdRenderLow,
    protoCmd_cmdRenderPrio,
    protocol,
};
use crate::schema::tile::identity::{LayerName, TileIdentity};
use crate::utility::type_meta::size_of_return_type;

use std::mem::size_of;


pub const MAX_LAYER_NAME_LEN: usize = 40;
pub const MAX_MIME_TYPE_LEN: usize = 40;
pub const MAX_OPTIONS_LEN: usize = 40;

pub enum RenderPriority {
}

pub enum RenderRequestVersion {
    Two = 2,
    Three = 3,
}

pub enum RenderRequestCommand {
    Render = protoCmd_cmdRender as isize,
    Dirty = protoCmd_cmdDirty as isize,
    RenderPriority = protoCmd_cmdRenderPrio as isize,
    RenderBulk = protoCmd_cmdRenderBulk as isize,
    RenderLow = protoCmd_cmdRenderLow as isize,
}

pub trait Constructable {
    fn new(
        version: RenderRequestVersion,
        command: RenderRequestCommand,
        layer: &LayerName,
        tile_id: &TileIdentity,
        extension: &str,
    ) -> protocol;
}

impl Constructable for protocol {
    fn new(
        version: RenderRequestVersion,
        command: RenderRequestCommand,
        layer: &LayerName,
        tile_id: &TileIdentity,
        extension: &str,
    ) -> protocol {
        let mut result = protocol {
            ver: version as std::os::raw::c_int,
            cmd: command as protoCmd,
            x: tile_id.x as std::os::raw::c_int,
            y: tile_id.y as std::os::raw::c_int,
            z: tile_id.z as std::os::raw::c_int,
            xmlname: [0; MAX_LAYER_NAME_LEN + 1],
            mimetype: [0; MAX_MIME_TYPE_LEN + 1],
            options: [0; MAX_OPTIONS_LEN + 1],
        };
        const _: () = assert!(
            size_of_return_type(|p: protocol| p.xmlname) > size_of::<LayerName>(),
            "LayerName is too long for Renderd request"
        );

        let layer_as_i8_slice = unsafe {
            // on x86_64 c_char is aliased to i8
            core::slice::from_raw_parts_mut(
                layer.as_u8().as_ptr() as *mut i8,
                layer.len(),
            )
        };
        result.xmlname.as_mut_slice().copy_from_slice(layer_as_i8_slice);

        let extension_as_i8_slice = unsafe {
            // on x86_64 c_char is aliased to i8
            core::slice::from_raw_parts_mut(
                extension.as_bytes().as_ptr() as *mut i8,
                extension.len(),
            )
        };
        result.mimetype.as_mut_slice().copy_from_slice(extension_as_i8_slice);
        result
    }
}
