#![allow(unused_unsafe)]

use crate::slippy::context::RequestContext;
use crate::slippy::error::{
    InvalidParameterError, ParseError
};
use crate::slippy::request::{
    BodyVariant, Header, Request, ServeTileRequestV2, ServeTileRequestV3
};

use crate::tile::config::{ LayerConfig, TileConfig, };

use scan_fmt::scan_fmt;

use std::ffi::CStr;
use std::option::Option;
use std::result::Result;
use std::string::String;


pub struct SlippyRequestParser;
impl SlippyRequestParser {
    pub fn parse(
        context: &RequestContext,
        config: &TileConfig,
        request_url: &str,
    ) -> Result<Option<Request>, ParseError> {
        debug!(context.get_host().record, "SlippyRequestParser::parse - start");
        // try match stats request
        if let Some(request) = StatisticsRequestParser::parse(context, config, request_url)? {
            return Ok(Some(request));
        }
        let parse_layer_request = LayerParserCombinator::or_else(
            DescribeLayerRequestParser::parse,
            LayerParserCombinator::or_else(
                ServeTileV3RequestParser::parse,
                ServeTileV2RequestParser::parse,
            )
        );
        for (layer, config) in &(context.get_host().tile_config.layers) {
            info!(
                context.get_host().record,
                "SlippyRequestParser::parse - comparing layer {} with base URL {} to uri",
                layer,
                request_url
            );
            if let Some(found) = request_url.find(&config.base_url) {
                let after_base = found + config.base_url.len();
                if let Some(layer_url) = request_url.get(after_base..) {
                    if let Some(request) = parse_layer_request(context, config, layer_url)? {
                        return Ok(Some(request));
                    }
                }
            };
        }
        info!(context.get_host().record, "SlippyRequestParser::parse - URL {} does not match any known request types", request_url);
        return Err(ParseError::Param(
            InvalidParameterError {
                param: String::from("uri"),
                value: request_url.to_string(),
                reason: "Does not match any known request types".to_string(),
            }
        ));
    }
}

struct LayerParserCombinator;
impl LayerParserCombinator {
    // TODO: remove the repeated trait bound once trait aliases is stable
    fn or_else<F, G>(
        func1: F,
        func2: G,
    ) -> impl Fn(&RequestContext, &LayerConfig, &str) -> Result<Option<Request>, ParseError>
    where
        F: Fn(&RequestContext, &LayerConfig, &str) -> Result<Option<Request>, ParseError>,
        G: Fn(&RequestContext, &LayerConfig, &str) -> Result<Option<Request>, ParseError>,
    {
        move |context, config, request_url| {
            if let Some(request) = func1(context, config, request_url)? {
                return Ok(Some(request));
            } else {
                return func2(context, config, request_url);
            }
        }
    }
}

struct StatisticsRequestParser;
impl StatisticsRequestParser {
    fn parse(
        context: &RequestContext,
        _config: &TileConfig,
        request_url: &str,
    ) -> Result<Option<Request>, ParseError> {
        let module_name = unsafe {
            CStr::from_ptr(crate::TILE_MODULE.name).to_str()?
        };
        let stats_uri = format!("/{}", module_name);
        if request_url.eq(&stats_uri) {
            info!(context.get_host().record, "StatisticsRequestParser::parse - matched ReportStatistics");
            return Ok(Some(Request {
                header: Header::new(
                    context.record,
                    context.connection.record,
                    context.get_host().record
                ),
                body: BodyVariant::ReportStatistics,
            }));
        } else {
            info!(context.get_host().record, "StatisticsRequestParser::parse - no match");
            return Ok(None);
        }
    }
}

struct DescribeLayerRequestParser;
impl DescribeLayerRequestParser {
    fn parse(
        context: &RequestContext,
        layer_config: &LayerConfig,
        request_url: &str,
    ) -> Result<Option<Request>, ParseError> {
        if request_url.eq_ignore_ascii_case("/tile-layer.json") {
            info!(context.get_host().record, "DescribeLayerRequestParser::parse - matched DescribeLayer");
            return Ok(Some(Request {
                header: Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &(layer_config.name),
                ),
                body: BodyVariant::DescribeLayer
            }));
        } else {
            info!(context.get_host().record, "DescribeLayerRequestParser::parse - no match");
            return Ok(None);
        }
    }
}

struct ServeTileV3RequestParser;
impl ServeTileV3RequestParser {
    fn parse(
        context: &RequestContext,
        layer_config: &LayerConfig,
        request_url: &str,
    ) -> Result<Option<Request>, ParseError> {
        // TODO: replace with a more modular parser that better handles with option and no option
        if !(layer_config.parameters_allowed) {
            return Ok(None);
        }

        // try match with option
        match scan_fmt!(
            request_url,
            "/{40[^/]}/{d}/{d}/{d}.{255[a-z]}/{10[^/]}",
            String, i32, i32, i32, String, String
        ) {
            Ok((parameter, x, y, z, extension, option)) => {
                info!(context.get_host().record, "ServeTileV3RequestParser::parse - matched ServeTileV3 with option");
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

        // try match no option
        match scan_fmt!(
            request_url,
            "/{40[^/]}/{d}/{d}/{d}.{255[a-z]}{///?/}",
            String, i32, i32, i32, String
        ) {
            Ok((parameter, x, y, z, extension)) => {
                info!(context.get_host().record, "ServeTileV3RequestParser::parse - matched ServeTileV3 no option");
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

        info!(context.get_host().record, "ServeTileV3RequestParser::parse - no match");
        return Ok(None);
    }
}

struct ServeTileV2RequestParser;
impl ServeTileV2RequestParser {
    fn parse(
        context: &RequestContext,
        layer_config: &LayerConfig,
        request_url: &str,
    ) -> Result<Option<Request>, ParseError> {
    // TODO: replace with a more modular parser that better handles with option and no option
    // try match with option
    match scan_fmt!(
        request_url,
        "/{d}/{d}/{d}.{255[a-z]}/{10[^/]}",
        i32, i32, i32, String, String
    ) {
        Ok((x, y, z, extension, option)) => {
            info!(context.get_host().record, "ServeTileV2RequestParser::parse - matched ServeTileV2 with option");
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

    // try match no option
    match scan_fmt!(
        request_url,
        "/{d}/{d}/{d}.{255[a-z]}{///?/}",
        i32, i32, i32, String
    ) {
        Ok((x, y, z, extension)) => {
            info!(context.get_host().record, "ServeTileV2RequestParser::parse - matched ServeTileV2 no option");
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
        info!(context.get_host().record, "ServeTileV2RequestParser::parse - no match");
        return Ok(None)
    }
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
            let expected_header = Header::new(
                context.record,
                context.connection.record,
                context.get_host().record
            );
            assert_eq!(expected_header, actual_request.header, "Wrong header generated");
            assert!(matches!(actual_request.body, BodyVariant::ReportStatistics));
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
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
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let config = &(context.get_host().tile_config);
            let request_url = context.uri;

            let actual_request = SlippyRequestParser::parse(context, config, request_url)?.unwrap();
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
