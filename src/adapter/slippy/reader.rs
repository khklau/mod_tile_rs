use crate::binding::apache2::get_module_name;
use crate::core::identifier::generate_id;
use crate::schema::apache2::config::{ LayerConfig, MAX_ZOOM_SERVER };
use crate::schema::core::processed::ProcessOutcome;
use crate::schema::http::request::HttpRequest;
use crate::schema::slippy::error::{
    InvalidParameterError, ReadError
};
use crate::schema::slippy::request::{
    BodyVariant, Header, MAX_EXTENSION_LEN,
    SlippyRequest, ServeTileRequestV2, ServeTileRequestV3
};
use crate::schema::slippy::result::ReadOutcome;
use crate::schema::tile::identity::LayerName;
use crate::adapter::slippy::interface::ReadContext;

use const_format::concatcp;
use scan_fmt::scan_fmt;

use std::string::String;


pub struct SlippyRequestReader;
impl SlippyRequestReader {
    pub fn read(
        context: &ReadContext,
        request: &HttpRequest,
    ) -> ReadOutcome {
        let request_url= request.uri;
        SlippyRequestParser::parse(&context, request, request_url)
    }
}

pub struct SlippyRequestParser;
impl SlippyRequestParser {
    pub fn parse(
        context: &ReadContext,
        request: &HttpRequest,
        request_url: &str,
    ) -> ReadOutcome {
        debug!(context.host().record, "SlippyRequestParser::parse - start");
        // try match stats request
        let stat_outcome = StatisticsRequestParser::parse(&context, request, request_url);
        if stat_outcome.is_processed() {
            return stat_outcome;
        }
        let parse_layer_request = LayerParserCombinator::try_else(
            DescribeLayerRequestParser::parse,
            LayerParserCombinator::try_else(
                ServeTileV3RequestParser::parse,
                ServeTileV2RequestParser::parse,
            )
        );
        for (layer, config) in &(context.module_config().layers) {
            info!(
                context.host().record,
                "SlippyRequestParser::parse - comparing layer {} with base URL {} to uri",
                layer,
                request_url
            );
            if let Some(found) = request_url.find(&config.base_url) {
                let after_base = found + config.base_url.len();
                if let Some(layer_trimmed_url) = request_url.get(after_base..) {
                    let layer_outcome = parse_layer_request(context, config, request, layer_trimmed_url);
                    if layer_outcome.is_processed() {
                        return layer_outcome;
                    }
                }
            };
        }
        info!(context.host().record, "SlippyRequestParser::parse - URL {} does not match any known request types", request_url);
        return ProcessOutcome::Ignored;
    }
}

struct LayerParserCombinator;
impl LayerParserCombinator {
    // TODO: remove the repeated trait bound once trait aliases is stable
    fn try_else<F, G>(
        func1: F,
        func2: G,
    ) -> impl Fn(&ReadContext, &LayerConfig, &HttpRequest, &str) -> ReadOutcome
    where
        F: Fn(&ReadContext, &LayerConfig, &HttpRequest, &str) -> ReadOutcome,
        G: Fn(&ReadContext, &LayerConfig, &HttpRequest, &str) -> ReadOutcome,
    {
        move |context, config, request, request_url| {
            let outcome = func1(context, config, request, request_url);
            if outcome.is_processed() {
                outcome
            } else {
                func2(context, config, request, request_url)
            }
        }
    }
}

struct StatisticsRequestParser;
impl StatisticsRequestParser {
    fn parse(
        context: &ReadContext,
        request: &HttpRequest,
        _request_url: &str,
    ) -> ReadOutcome {
        let module_name = get_module_name();
        let stats_uri = format!("/{}", module_name);
        if request.uri.eq(&stats_uri) {
            info!(context.host().record, "StatisticsRequestParser::parse - matched ReportStatistics");
            ProcessOutcome::Processed(
                Ok(
                    SlippyRequest {
                        header: Header {
                            layer: LayerName::new(),
                            request_id: generate_id(),
                            uri: request.uri.to_string(),
                            received_timestamp: request.received_time.clone(),
                        },
                        body: BodyVariant::ReportStatistics,
                    }
                )
            )
        } else {
            info!(context.host().record, "StatisticsRequestParser::parse - no match");
            ProcessOutcome::Ignored
        }
    }
}

struct DescribeLayerRequestParser;
impl DescribeLayerRequestParser {
    fn parse(
        context: &ReadContext,
        layer_config: &LayerConfig,
        request: &HttpRequest,
        layer_trimmed_url: &str,
    ) -> ReadOutcome {
        if layer_trimmed_url.eq_ignore_ascii_case("/tile-layer.json") {
            info!(context.host().record, "DescribeLayerRequestParser::parse - matched DescribeLayer");
            ProcessOutcome::Processed(
                Ok(
                    SlippyRequest {
                        header: Header {
                            layer: layer_config.name.clone(),
                            request_id: generate_id(),
                            uri: request.uri.to_string(),
                            received_timestamp: request.received_time.clone(),
                        },
                        body: BodyVariant::DescribeLayer
                    }
                )
            )
        } else {
            info!(context.host().record, "DescribeLayerRequestParser::parse - no match");
            ProcessOutcome::Ignored
        }
    }
}

struct ServeTileV3RequestParser;
impl ServeTileV3RequestParser {
    fn parse(
        context: &ReadContext,
        layer_config: &LayerConfig,
        request: &HttpRequest,
        layer_trimmed_url: &str,
    ) -> ReadOutcome {
        // TODO: replace with a more modular parser that better handles with option and no option
        let has_parameter = match scan_fmt!(
            layer_trimmed_url,
            "/{40[^0-9/]}/",
            String
        ) {
            Ok(_) => true,
            Err(_) => false,
        };
        if !has_parameter {
            return ProcessOutcome::Ignored;
        } else if has_parameter && !(layer_config.parameters_allowed) {
            return ProcessOutcome::Processed(
                Err(
                    ReadError::Param(
                        InvalidParameterError {
                            param: String::from("uri"),
                            value: request.uri.to_string(),
                            reason: "Request has parameter but parameterize_style not set in config".to_string(),
                        }
                    )
                )
            );
        }

        // try match with option
        match scan_fmt!(
            layer_trimmed_url,
            concatcp!("/{40[^/]}/{d}/{d}/{d}.{", MAX_EXTENSION_LEN, "[a-z]}/{10[^/]}"),
            String, i32, i32, i32, String, String
        ) {
            Ok((parameter, x, y, z, extension, option)) => {
                info!(context.host().record, "ServeTileV3RequestParser::parse - matched ServeTileV3 with option");
                if z <= MAX_ZOOM_SERVER as i32 {
                    return ProcessOutcome::Processed(
                        Ok(
                            SlippyRequest {
                                header: Header {
                                    layer: layer_config.name.clone(),
                                    request_id: generate_id(),
                                    uri: request.uri.to_string(),
                                    received_timestamp: request.received_time.clone(),
                                },
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
                            }
                        )
                    );
                } else {
                    return ProcessOutcome::Processed(
                        Err(
                            ReadError::Param(
                                InvalidParameterError {
                                    param: String::from("z"),
                                    value: request.uri.to_string(),
                                    reason: format!("Request parameter z {} exceeds the allowed limit {}", z, MAX_ZOOM_SERVER),
                                }
                            )
                        )
                    );
                }
            },
            Err(_) => ()
        }

        // try match no option
        match scan_fmt!(
            layer_trimmed_url,
            concatcp!("/{40[^/]}/{d}/{d}/{d}.{", MAX_EXTENSION_LEN, "[a-z]}{///?/}"),
            String, i32, i32, i32, String
        ) {
            Ok((parameter, x, y, z, extension)) => {
                info!(context.host().record, "ServeTileV3RequestParser::parse - matched ServeTileV3 no option");
                if z <= MAX_ZOOM_SERVER as i32 {
                    return ProcessOutcome::Processed(
                        Ok(
                            SlippyRequest {
                                header: Header {
                                    layer: layer_config.name.clone(),
                                    request_id: generate_id(),
                                    uri: request.uri.to_string(),
                                    received_timestamp: request.received_time.clone(),
                                },
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
                            }
                        )
                    );
                } else {
                    return ProcessOutcome::Processed(
                        Err(
                            ReadError::Param(
                            InvalidParameterError {
                                param: String::from("z"),
                                value: request.uri.to_string(),
                                reason: format!("Request parameter z {} exceeds the allowed limit {}", z, MAX_ZOOM_SERVER),
                            }
                            )
                        )
                    );
                }
            },
            Err(_) => ()
        }

        info!(context.host().record, "ServeTileV3RequestParser::parse - no match");
        return ProcessOutcome::Ignored;
    }
}

struct ServeTileV2RequestParser;
impl ServeTileV2RequestParser {
    fn parse(
        context: &ReadContext,
        layer_config: &LayerConfig,
        request: &HttpRequest,
        layer_trimmed_url: &str,
    ) -> ReadOutcome {
    // TODO: replace with a more modular parser that better handles with option and no option
    // try match with option
    match scan_fmt!(
        layer_trimmed_url,
        concatcp!("/{d}/{d}/{d}.{", MAX_EXTENSION_LEN, "[a-z]}/{10[^/]}"),
        i32, i32, i32, String, String
    ) {
        Ok((x, y, z, extension, option)) => {
            if z <= MAX_ZOOM_SERVER as i32 {
                info!(context.host().record, "ServeTileV2RequestParser::parse - matched ServeTileV2 with option");
                return ProcessOutcome::Processed(
                    Ok(
                        SlippyRequest {
                            header: Header {
                                layer: layer_config.name.clone(),
                                request_id: generate_id(),
                                uri: request.uri.to_string(),
                                received_timestamp: request.received_time.clone(),
                            },
                            body: BodyVariant::ServeTileV2(
                                ServeTileRequestV2 {
                                    x,
                                    y,
                                    z,
                                    extension,
                                    option: Some(option),
                                }
                            ),
                        }
                    )
                );
            } else {
                return ProcessOutcome::Processed(
                    Err(
                        ReadError::Param(
                            InvalidParameterError {
                                param: String::from("z"),
                                value: request.uri.to_string(),
                                reason: format!("Request parameter z {} exceeds the allowed limit {}", z, MAX_ZOOM_SERVER),
                            }
                        )
                    )
                );
            }
        },
        Err(_) => ()
    }

    // try match no option
    match scan_fmt!(
        layer_trimmed_url,
        concatcp!("/{d}/{d}/{d}.{", MAX_EXTENSION_LEN, "[a-z]}{///?/}"),
        i32, i32, i32, String
    ) {
        Ok((x, y, z, extension)) => {
            if z <= MAX_ZOOM_SERVER as i32 {
                info!(context.host().record, "ServeTileV2RequestParser::parse - matched ServeTileV2 no option");
                return ProcessOutcome::Processed(
                    Ok(
                        SlippyRequest {
                            header: Header {
                                layer: layer_config.name.clone(),
                                request_id: generate_id(),
                                uri: request.uri.to_string(),
                                received_timestamp: request.received_time.clone(),
                            },
                            body: BodyVariant::ServeTileV2(
                                ServeTileRequestV2 {
                                    x,
                                    y,
                                    z,
                                    extension,
                                    option: None,
                                }
                            )
                        }
                    )
                );
            } else {
                return ProcessOutcome::Processed(
                    Err(
                        ReadError::Param(
                            InvalidParameterError {
                                param: String::from("z"),
                                value: request.uri.to_string(),
                                reason: format!("Request parameter z {} exceeds the allowed limit {}", z, MAX_ZOOM_SERVER),
                            }
                        )
                    )
                );
            }
        },
        Err(_) => ()
    }
        info!(context.host().record, "ServeTileV2RequestParser::parse - no match");
        return ProcessOutcome::Ignored;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::apache2::config::ModuleConfig;
    use crate::schema::apache2::virtual_host::VirtualHost;
    use crate::core::memory::PoolStored;
    use crate::framework::apache2::context::HostContext;
    use crate::framework::apache2::record::test_utils::with_request_rec;
    use chrono::Utc;
    use std::boxed::Box;
    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn test_parse_report_mod_stats() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let module_config = ModuleConfig::new();
            let uri = CString::new("/mod_tile_rs")?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            assert_eq!(BodyVariant::ReportStatistics, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_describe_layer() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            let expected_layer = layer_name.clone();
            assert_eq!(BodyVariant::DescribeLayer, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_with_option() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
            let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            layer_config.parameters_allowed = true;
            let uri = CString::new(format!("{}/foo/7/8/9.png/bar", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            let expected_layer = layer_name.clone();
            let expected_body = BodyVariant::ServeTileV3(
                ServeTileRequestV3 {
                    parameter: String::from("foo"),
                    x: 7,
                    y: 8,
                    z: 9,
                    extension: String::from("png"),
                    option: Some(String::from("bar")),
                }
            );
            assert_eq!(expected_body, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_with_invalid_zoom_param() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
           let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            layer_config.parameters_allowed = true;
            let uri = CString::new(format!("{}/foo/7/8/999.png/bar", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            match SlippyRequestParser::parse(&context, &request, request_url).expect_processed().unwrap_err() {
                ReadError::Param(err) => {
                    assert_eq!("z", err.param, "Did not identify zoom as the invalid parameter");
                },
                _ => {
                    panic!("Expected InvalidParameterError in result");
                }
            }
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_no_option_with_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
           let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            layer_config.parameters_allowed = true;
            let uri = CString::new(format!("{}/foo/7/8/9.png/", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            let expected_layer = layer_name.clone();
            let expected_body = BodyVariant::ServeTileV3(
                ServeTileRequestV3 {
                    parameter: String::from("foo"),
                    x: 7,
                    y: 8,
                    z: 9,
                    extension: String::from("png"),
                    option: None,
                }
            );
            assert_eq!(expected_body, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v3_no_option_no_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
           let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            layer_config.parameters_allowed = true;
            let uri = CString::new(format!("{}/foo/7/8/9.png", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            let expected_layer = layer_name.clone();
            let expected_body = BodyVariant::ServeTileV3(
                ServeTileRequestV3 {
                    parameter: String::from("foo"),
                    x: 7,
                    y: 8,
                    z: 9,
                    extension: String::from("png"),
                    option: None,
                }
            );
            assert_eq!(expected_body, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_with_option() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
           let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            let uri = CString::new(format!("{}/1/2/3.jpg/blah", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            let expected_layer = layer_name.clone();
            let expected_body = BodyVariant::ServeTileV2(
                ServeTileRequestV2 {
                    x: 1,
                    y: 2,
                    z: 3,
                    extension: String::from("jpg"),
                    option: Some(String::from("blah")),
                }
            );
            assert_eq!(expected_body, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_with_invalid_zoom_param() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
           let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            let uri = CString::new(format!("{}/1/2/999.jpg/blah", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            match SlippyRequestParser::parse(&context, &request, request_url).expect_processed().unwrap_err() {
                ReadError::Param(err) => {
                    assert_eq!("z", err.param, "Did not identify zoom as the invalid parameter");
                },
                _ => {
                    panic!("Expected InvalidParameterError in result");
                }
            }
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_no_option_with_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
           let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            let uri = CString::new(format!("{}/1/2/3.jpg/", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            let expected_layer = layer_name.clone();
            let expected_body = BodyVariant::ServeTileV2(
                ServeTileRequestV2 {
                    x: 1,
                    y: 2,
                    z: 3,
                    extension: String::from("jpg"),
                    option: None,
                }
            );
            assert_eq!(expected_body, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }

    #[test]
    fn test_parse_serve_tile_v2_no_option_no_ending_forward_slash() -> Result<(), Box<dyn Error>> {
        with_request_rec(|record| {
           let layer_name = LayerName::from("default");
            let mut module_config = ModuleConfig::new();
            let layer_config = module_config.layers.get_mut(&layer_name).unwrap();
            let uri = CString::new(format!("{}/1/2/3.jpg", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = ReadContext {
                host_context: HostContext {
                    module_config: &module_config,
                    host: VirtualHost::find_or_allocate_new(record)?,
                }
            };
            let request = HttpRequest::new(
                uri.as_c_str().to_str()?,
                Utc::now(),
                record,
            );
            let request_url= request.uri;

            let actual_request = SlippyRequestParser::parse(&context, &request, request_url).expect_processed()?;
            let expected_layer = layer_name.clone();
            let expected_body = BodyVariant::ServeTileV2(
                ServeTileRequestV2 {
                    x: 1,
                    y: 2,
                    z: 3,
                    extension: String::from("jpg"),
                    option: None,
                }
            );
            assert_eq!(expected_body, actual_request.body, "Incorrect parsing");
            Ok(())
        })
    }
}
