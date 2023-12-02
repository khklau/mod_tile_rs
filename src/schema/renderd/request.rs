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
use crate::schema::renderd::error::InvalidParameterError;
use crate::core::type_meta::size_of_return_type;

use const_format::concatcp;

use std::mem::size_of;
use std::result::Result;


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
    ) -> Result<protocol, InvalidParameterError>;
}

impl Constructable for protocol {
    fn new(
        version: RenderRequestVersion,
        command: RenderRequestCommand,
        layer: &LayerName,
        tile_id: &TileIdentity,
        extension: &str,
    ) -> Result<protocol, InvalidParameterError> {
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

        const _LAYER_NAME_LIMIT: usize = size_of_return_type(|p: protocol| p.xmlname);
        const _: () = assert!(
            size_of::<LayerName>() < _LAYER_NAME_LIMIT,
            "{}",
            concatcp!("LayerName must be less than ", _LAYER_NAME_LIMIT)
        );
        let layer_as_i8_slice = unsafe {
            // on x86_64 c_char is aliased to i8
            core::slice::from_raw_parts_mut(
                layer.as_u8().as_ptr() as *mut i8,
                layer.len(),
            )
        };
        result.xmlname.as_mut_slice()[..layer.len()].copy_from_slice(layer_as_i8_slice);
        result.xmlname.as_mut_slice()[layer.len()] = 0;  // C string null terminator

        const MIME_TYPE_LIMIT: usize = size_of_return_type(|p: protocol| p.mimetype);
        if extension.len() >= MIME_TYPE_LIMIT {
            return Err(
                InvalidParameterError {
                    param: "mimetype".to_string(),
                    value: extension.to_string(),
                    reason: format!("Mimetype parameter must be less than {}", MIME_TYPE_LIMIT),
                }
            )
        }
        let extension_as_i8_slice = unsafe {
            // on x86_64 c_char is aliased to i8
            core::slice::from_raw_parts_mut(
                extension.as_bytes().as_ptr() as *mut i8,
                extension.len(),
            )
        };
        result.mimetype.as_mut_slice()[..extension.len()].copy_from_slice(extension_as_i8_slice);
        result.mimetype.as_mut_slice()[extension.len()] = 0;  // C string null terminator
        Ok(result)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::error::Error;
    use std::os::raw::c_char;

    #[test]
    fn test_new_basic_protocol() -> Result<(), Box<dyn Error>> {
        let version = RenderRequestVersion::Three;
        let command = RenderRequestCommand::RenderLow;
        let layer = LayerName::make("1234567890123456");
        let tile_id = TileIdentity {
            x: 1,
            y: 2,
            z: 3,
            layer: layer.clone(),
        };
        let extension = "1234567890123456789012345678901234567890";
        let value = protocol::new(
            version,
            command,
            &layer,
            &tile_id,
            extension,
        )?;
        let actual_layer = unsafe { CStr::from_ptr(value.xmlname.as_ptr() as *const c_char) };
        let actual_extension= unsafe { CStr::from_ptr(value.mimetype.as_ptr() as *const c_char) };
        assert!(layer.len() == actual_layer.to_bytes().len(), "Copied layer name has wrong length");
        assert!(extension.len() == actual_extension.to_bytes().len(), "Copied extension has wrong length");
        Ok(())
    }

    #[test]
    fn test_new_protocol_with_invalid_extension() -> Result<(), Box<dyn Error>> {
        let version = RenderRequestVersion::Three;
        let command = RenderRequestCommand::RenderLow;
        let layer = LayerName::make("1234567890123456");
        let tile_id = TileIdentity {
            x: 1,
            y: 2,
            z: 3,
            layer: layer.clone(),
        };
        let extension = "12345678901234567890123456789012345678901";
        let result = protocol::new(
            version,
            command,
            &layer,
            &tile_id,
            extension,
        );
        assert!(result.is_err(), "Protocol constructor did not reject invallid extension argument");
        Ok(())
    }
}
