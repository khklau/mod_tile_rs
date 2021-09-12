#![allow(unused_unsafe)]

use crate::slippy::context::RequestContext;
use crate::slippy::error::{
    InvalidParameterError, ParseError
};
use crate::slippy::request::{
    BodyVariant, Header, Request, ServeTileRequestV2, ServeTileRequestV3
};
use crate::slippy::traits::RequestParser;

use crate::apache2::bindings::{
    DECLINED, HTTP_INTERNAL_SERVER_ERROR, OK,
    request_rec,
};
use crate::tile::config::{ LayerConfig, TileConfig, };

use scan_fmt::scan_fmt;

use std::ffi::CStr;
use std::os::raw::c_int;
use std::option::Option;
use std::ptr;
use std::result::Result;
use std::string::String;


struct Parser<'c> {
    pub config: &'c TileConfig,
}

impl<'c> RequestParser for Parser<'c> {

    fn parse(&self, context: &RequestContext) -> Result<Option<Request>, ParseError> {
        Ok(None)
    }

}


#[no_mangle]
pub extern "C" fn parse(record_ptr: *mut request_rec) -> c_int {
    if record_ptr == ptr::null_mut() {
        return HTTP_INTERNAL_SERVER_ERROR as c_int;
    } else {
        unsafe {
            let record = &mut *record_ptr;
            info!(record.server, "slippy::request::parse - start");
            let context = match RequestContext::find_or_create(record) {
                Ok(context) => context,
                Err(_) => return HTTP_INTERNAL_SERVER_ERROR as c_int,
            };
            match parse_request(context) {
                Ok(result) => {
                    match result {
                        Some(request) => {
                            context.request = Some(request);
                            info!(record.server, "slippy::request::parse - finish");
                            return OK as c_int;
                        },
                        None => {
                            return DECLINED as c_int;
                        },
                    }
                },
                Err(err) => match err {
                    ParseError::Param(err) => {
                        info!(record.server, "Parameter {} error: {}", err.param, err.reason);
                        return DECLINED as c_int;
                    },
                    ParseError::Io(why) => {
                        info!(record.server, "IO error: {}", why);
                        return HTTP_INTERNAL_SERVER_ERROR as c_int;
                    },
                    ParseError::Utf8(why) => {
                        info!(record.server, "UTF8 error: {}", why);
                        return HTTP_INTERNAL_SERVER_ERROR as c_int;
                    },
                },
            }
        }
    }
}

fn parse_request(context: &RequestContext) -> Result<Option<Request>, ParseError> {
    info!(context.get_host().record, "slippy::request::parse_request - uri={}", context.uri);

    // try match stats request
    let module_name = unsafe {
        CStr::from_ptr(crate::TILE_MODULE.name).to_str()?
    };
    let stats_uri = format!("/{}", module_name);
    if context.uri.eq(&stats_uri) {
        info!(context.get_host().record, "slippy::request::parse_layer_request - parsed ReportModStats");
        return Ok(Some(Request {
            header: Header::new(
                context.record,
                context.connection.record,
                context.get_host().record
            ),
            body: BodyVariant::ReportModStats,
        }));
    }

    for (layer, config) in &(context.get_host().tile_config.layers) {
        info!(
            context.get_host().record,
            "slippy::request::parse_request - comparing layer {} with base URL {} to uri",
            layer,
            config.base_url,
        );
        if let Some(found) = context.uri.find(&config.base_url) {
            let after_base = found + config.base_url.len();
            if let Some(request_url) = context.uri.get(after_base..) {
                if let Some(request) = parse_layer_request(context, config, request_url)? {
                    return Ok(Some(request));
                }
            }
        };
    }
    return Ok(None);
}

fn parse_layer_request(
    context: &RequestContext,
    layer_config: &LayerConfig,
    request_url: &str,
) -> Result<Option<Request>, ParseError> {
    info!(
        context.get_host().record,
        "slippy::request;::parse_layer_request - layer={}, request_url={}",
        layer_config.name,
        request_url,
    );

    // try match the JSON layer description request
    if request_url.eq_ignore_ascii_case("/tile-layer.json") {
        return Ok(Some(Request {
            header: Header::new_with_layer(
                context.record,
                context.connection.record,
                context.get_host().record,
                &(layer_config.name),
            ),
            body: BodyVariant::DescribeLayer
        }));
    }

    if layer_config.parameters_allowed {
        // try match ServeTileV3 with option
        match scan_fmt!(
            request_url,
            "/{40[^/]}/{d}/{d}/{d}.{255[a-z]}/{10[^/]}",
            String, i32, i32, i32, String, String
        ) {
            Ok((parameter, x, y, z, extension, option)) => {
                info!(context.get_host().record, "slippy::request::parse_layer_request - parsed ServeTileV3 with option");
                return Ok(Some(Request {
                    header: Header::new_with_layer(
                        context.record,
                        context.connection.record,
                        context.get_host().record,
                        &(layer_config.name),
                    ),
                    body: BodyVariant::ServeTileV3(
                        ServeTileRequestV3 {
                            parameter,
                            x,
                            y,
                            z,
                            extension,
                            option: Some(option)
                        }
                    ),
                }));
            },
            Err(_) => ()
        }

        // try match ServeTileV3 no option
        match scan_fmt!(
            request_url,
            "/{40[^/]}/{d}/{d}/{d}.{255[a-z]}{///?/}",
            String, i32, i32, i32, String
        ) {
            Ok((parameter, x, y, z, extension)) => {
                info!(context.get_host().record, "slippy::request::parse_layer_request - parsed ServeTileV3 no option");
                return Ok(Some(Request {
                    header: Header::new_with_layer(
                        context.record,
                        context.connection.record,
                        context.get_host().record,
                        &(layer_config.name),
                    ),
                    body: BodyVariant::ServeTileV3(
                        ServeTileRequestV3 {
                            parameter,
                            x,
                            y,
                            z,
                            extension,
                            option: None,
                        }
                    ),
                }));
            },
            Err(_) => ()
        }
    }

    // try match ServeTileV2 with option
    match scan_fmt!(
        request_url,
        "/{d}/{d}/{d}.{255[a-z]}/{10[^/]}",
        i32, i32, i32, String, String
    ) {
        Ok((x, y, z, extension, option)) => {
            info!(context.get_host().record, "slippy::request::parse_layer_request - parsed ServeTileV2");
            return Ok(Some(Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &(layer_config.name),
                ),
                body: BodyVariant::ServeTileV2(
                    ServeTileRequestV2 {
                        x,
                        y,
                        z,
                        extension,
                        option: Some(option),
                    }
                ),
            }));
        },
        Err(_) => ()
    }

    // try match ServeTileV2 no option
    match scan_fmt!(
        request_url,
        "/{d}/{d}/{d}.{255[a-z]}{///?/}",
        i32, i32, i32, String
    ) {
        Ok((x, y, z, extension)) => {
            info!(context.get_host().record, "slippy::request::parse_layer_request - parsed ServeTileV2");
            return Ok(Some(Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &(layer_config.name),
                ),
                body: BodyVariant::ServeTileV2(
                    ServeTileRequestV2 {
                        x,
                        y,
                        z,
                        extension,
                        option: None,
                    }
                )
            }));
        },
        Err(_) => ()
    }

    info!(context.get_host().record, "slippy::request::parse_layer_request - URI {} does not match any known request types", request_url);
    return Err(ParseError::Param(
        InvalidParameterError {
            param: "uri".to_string(),
            value: request_url.to_string(),
            reason: "Does not match any known request types".to_string(),
        }
    ));
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::slippy::context::test_utils::with_request_rec;
    use crate::tile::config::TileConfig;
    use std::boxed::Box;
    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn test_parse_report_mod_stats() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let tile_config = TileConfig::new();
            let uri = CString::new("/mod_tile_rs")?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_header = Header::new(
                context.record,
                context.connection.record,
                context.get_host().record
            );
            assert_eq!(expected_header, actual_request.header, "Wrong header generated");
            assert!(matches!(actual_request.body, BodyVariant::ReportModStats));
            Ok(())
        })
    }

    #[test]
    fn test_parse_describe_layer() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = "default";
            let mut tile_config = TileConfig::new();
            let layer_config = tile_config.layers.get_mut(layer_name).unwrap();
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_layer = String::from(layer_name);
            let expected_request = Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &expected_layer,
                ),
                body: BodyVariant::DescribeLayer,
            };
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_with_option() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = "default";
            let mut tile_config = TileConfig::new();
            let layer_config = tile_config.layers.get_mut(layer_name).unwrap();
            layer_config.parameters_allowed = true;
            let uri = CString::new(format!("{}/foo/7/8/9.png/bar", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_layer = String::from(layer_name);
            let expected_request = Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &expected_layer,
                ),
                body: BodyVariant::ServeTileV3(
                    ServeTileRequestV3 {
                        parameter: String::from("foo"),
                        x: 7,
                        y: 8,
                        z: 9,
                        extension: String::from("png"),
                        option: Some(String::from("bar")),
                    }
                ),
            };
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_no_option_with_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = "default";
            let mut tile_config = TileConfig::new();
            let layer_config = tile_config.layers.get_mut(layer_name).unwrap();
            layer_config.parameters_allowed = true;
            let uri = CString::new(format!("{}/foo/7/8/9.png/", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_layer = String::from(layer_name);
            let expected_request = Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &expected_layer,
                ),
                body: BodyVariant::ServeTileV3(
                    ServeTileRequestV3 {
                        parameter: String::from("foo"),
                        x: 7,
                        y: 8,
                        z: 9,
                        extension: String::from("png"),
                        option: None,
                    }
                ),
            };
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_no_option_no_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = "default";
            let mut tile_config = TileConfig::new();
            let layer_config = tile_config.layers.get_mut(layer_name).unwrap();
            layer_config.parameters_allowed = true;
            let uri = CString::new(format!("{}/foo/7/8/9.png", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_layer = String::from(layer_name);
            let expected_request = Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &expected_layer,
                ),
                body: BodyVariant::ServeTileV3(
                    ServeTileRequestV3 {
                        parameter: String::from("foo"),
                        x: 7,
                        y: 8,
                        z: 9,
                        extension: String::from("png"),
                        option: None,
                    }
                ),
            };
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_with_option() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = "default";
            let mut tile_config = TileConfig::new();
            let layer_config = tile_config.layers.get_mut(layer_name).unwrap();
            let uri = CString::new(format!("{}/1/2/3.jpg/blah", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_layer = String::from(layer_name);
            let expected_request = Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &expected_layer,
                ),
                body: BodyVariant::ServeTileV2(
                    ServeTileRequestV2 {
                        x: 1,
                        y: 2,
                        z: 3,
                        extension: String::from("jpg"),
                        option: Some(String::from("blah")),
                    }
                ),
            };
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_no_option_with_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = "default";
            let mut tile_config = TileConfig::new();
            let layer_config = tile_config.layers.get_mut(layer_name).unwrap();
            let uri = CString::new(format!("{}/1/2/3.jpg/", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_layer = String::from(layer_name);
            let expected_request = Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &expected_layer,
                ),
                body: BodyVariant::ServeTileV2(
                    ServeTileRequestV2 {
                        x: 1,
                        y: 2,
                        z: 3,
                        extension: String::from("jpg"),
                        option: None,
                    }
                ),
            };
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_no_option_no_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = "default";
            let mut tile_config = TileConfig::new();
            let layer_config = tile_config.layers.get_mut(layer_name).unwrap();
            let uri = CString::new(format!("{}/1/2/3.jpg", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, tile_config)?;
            let actual_request = parse_request(context)?.unwrap();
            let expected_layer = String::from(layer_name);
            let expected_request = Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &expected_layer,
                ),
                body: BodyVariant::ServeTileV2(
                    ServeTileRequestV2 {
                        x: 1,
                        y: 2,
                        z: 3,
                        extension: String::from("jpg"),
                        option: None,
                    }
                ),
            };
            assert_eq!(expected_request, actual_request, "Incorrect parsing");
            Ok(())
        })
    }
}
