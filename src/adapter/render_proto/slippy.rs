use crate::binding::renderd_protocol::{protoCmd, protocol, protocol_v2};
use crate::schema::renderd::error::InvalidParameterError;
use crate::schema::slippy::request::{
    Header,
    ServeTileRequestV2,
    ServeTileRequestV3,
};
use crate::schema::tile::identity::LayerName;
use crate::core::type_meta::size_of_return_type;
use crate::schema::renderd::request::{
    MAX_LAYER_NAME_LEN,
    MAX_MIME_TYPE_LEN,
    MAX_OPTIONS_LEN,
    RenderRequest,
    RenderRequestCommand,
    RenderRequestVersion,
};
use crate::adapter::render_proto::interface::{
    RenderProtoContext,
    ToRenderProto,
};

use std::mem::size_of;
use std::result::Result;


impl ToRenderProto for ServeTileRequestV2 {
    fn to_render_proto(
        &self,
        context: &RenderProtoContext,
        header: &Header,
    ) -> Result<RenderRequest, InvalidParameterError> {
        // TODO: calculate rendering priority
        let mut result = protocol_v2 {
            ver: RenderRequestVersion::Two as std::os::raw::c_int,
            cmd: RenderRequestCommand::Render as protoCmd,
            x: self.x as std::os::raw::c_int,
            y: self.y as std::os::raw::c_int,
            z: self.z as std::os::raw::c_int,
            xmlname: [0; MAX_LAYER_NAME_LEN + 1],
        };

        const _: () = protocol_v2::ASSERT_LAYER_NAME_FITS;
        set_xmlname(header, &mut result)?;

        Ok(
            RenderRequest::V2(result)
        )
    }
}

impl ToRenderProto for ServeTileRequestV3 {
    fn to_render_proto(
        &self,
        context: &RenderProtoContext,
        header: &Header,
    ) -> Result<RenderRequest, InvalidParameterError> {
        let mut result = protocol {
            ver: RenderRequestVersion::Three as std::os::raw::c_int,
            cmd: RenderRequestCommand::Render as protoCmd,
            x: self.x as std::os::raw::c_int,
            y: self.y as std::os::raw::c_int,
            z: self.z as std::os::raw::c_int,
            xmlname: [0; MAX_LAYER_NAME_LEN + 1],
            mimetype: [0; MAX_MIME_TYPE_LEN + 1],
            options: [0; MAX_OPTIONS_LEN + 1],
        };

        const _: () = protocol::ASSERT_LAYER_NAME_FITS;
        set_xmlname(header, &mut result)?;
        set_mimetype(self, &mut result)?;
        set_options(self, &mut result)?;

        Ok(
            RenderRequest::V3(result)
        )
    }
}

trait HasXmlNameField {
    const SIZE_OF_FIELD: usize;
    const ASSERT_LAYER_NAME_FITS: () = assert!(
        size_of::<LayerName>() < Self::SIZE_OF_FIELD,
        "LayerName does not fit",
    );

    fn xmlname_as_mut_slice(&mut self) -> &mut[i8];
}

impl HasXmlNameField for protocol_v2 {
    const SIZE_OF_FIELD: usize = size_of_return_type(|p: Self| p.xmlname);

    fn xmlname_as_mut_slice(&mut self) -> &mut[i8] {
        self.xmlname.as_mut_slice()
    }
}

impl HasXmlNameField for protocol {
    const SIZE_OF_FIELD: usize = size_of_return_type(|p: Self| p.xmlname);

    fn xmlname_as_mut_slice(&mut self) -> &mut[i8] {
        self.xmlname.as_mut_slice()
    }
}

fn set_xmlname<P: HasXmlNameField>(
    header: &Header,
    output: &mut P,
) -> Result<(), InvalidParameterError> {
    let layer_as_i8_slice = unsafe {
        // on x86_64 c_char is aliased to i8
        core::slice::from_raw_parts_mut(
            header.layer.as_u8().as_ptr() as *mut i8,
            header.layer.len(),
        )
    };
    output.xmlname_as_mut_slice()[..header.layer.len()].copy_from_slice(layer_as_i8_slice);
    output.xmlname_as_mut_slice()[header.layer.len()] = 0;  // C string null terminator
    Ok(())
}

fn set_mimetype(
    from: &ServeTileRequestV3,
    to: &mut protocol,
) -> Result<(), InvalidParameterError> {
    const _MIME_TYPE_LIMIT: usize = size_of_return_type(|p: protocol| p.mimetype);
    if from.extension.len() >= _MIME_TYPE_LIMIT {
        return Err(
            InvalidParameterError {
                param: "mimetype".to_string(),
                value: from.extension.to_string(),
                reason: format!("Mimetype parameter must be less than {}", _MIME_TYPE_LIMIT),
            }
        )
    }
    let extension_as_i8_slice = unsafe {
        // on x86_64 c_char is aliased to i8
        core::slice::from_raw_parts_mut(
            from.extension.as_bytes().as_ptr() as *mut i8,
            from.extension.len(),
        )
    };
    to.mimetype.as_mut_slice()[..from.extension.len()].copy_from_slice(extension_as_i8_slice);
    to.mimetype.as_mut_slice()[from.extension.len()] = 0;  // C string null terminator
    Ok(())
}

fn set_options(
    from: &ServeTileRequestV3,
    to: &mut protocol,
) -> Result<(), InvalidParameterError> {
    if let Some(options_value) = &from.option {
        const _OPTIONS_LIMIT: usize = size_of_return_type(|p: protocol| p.options);
        if options_value.len() >= _OPTIONS_LIMIT {
            return Err(
                InvalidParameterError {
                    param: "options".to_string(),
                    value: options_value.clone(),
                    reason: format!("Options parameter must be less than {}", _OPTIONS_LIMIT),
                }
            )
        }
        let options_as_i8_slice = unsafe {
            // on x86_64 c_char is aliased to i8
            core::slice::from_raw_parts_mut(
                from.extension.as_bytes().as_ptr() as *mut i8,
                from.extension.len(),
            )
        };
        to.options.as_mut_slice()[..options_value.len()].copy_from_slice(options_as_i8_slice);
        to.options.as_mut_slice()[options_value.len()] = 0;  // C string null terminator
    }
    Ok(())
}
