use crate::schema::handler::context::HandleContext;
use crate::interface::handler::RequestHandler;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::config::ModuleConfig;

use chrono::Utc;
use mime;

use std::string::String;


pub struct DescriptionHandler { }
impl RequestHandler for DescriptionHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &request::Request,
    ) -> HandleRequestResult {
        let before_timestamp = Utc::now();
        let layer = match request.body {
            request::BodyVariant::DescribeLayer => &request.header.layer,
            _ => {
                return HandleRequestResult {
                    before_timestamp,
                    after_timestamp: Utc::now(),
                    result: Ok(HandleOutcome::NotHandled),
                }
            },
        };
        let description = describe(context.request_context.get_config(), layer);
        let response = response::Response {
            header: response::Header::new(
                context.request_context.record,
                context.request_context.connection.record,
                context.request_context.get_host().record,
                &mime::APPLICATION_JSON,
            ),
            body: response::BodyVariant::Description(description),
        };
        let after_timestamp = Utc::now();
        return HandleRequestResult {
            before_timestamp,
            after_timestamp,
            result: Ok(HandleOutcome::Handled(response)),
        };
    }
}

fn describe(config: &ModuleConfig, layer: &String) -> response::Description {
    let layer_config = &config.layers[layer];
    let mut value = response::Description {
        tilejson: "2.0.0",
        schema: "xyz",
        name: layer_config.name.clone(),
        description: layer_config.description.clone(),
        attribution: layer_config.attribution.clone(),
        minzoom: layer_config.min_zoom,
        maxzoom: layer_config.max_zoom,
        tiles: Vec::new(),
    };
    value.tiles.push(
        format!(
            "{}{}/{{z}}/{{x}}/{{y}}.{}",
            layer_config.host_name,
            layer_config.base_url,
            layer_config.file_extension,
        )
    );
    value
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::apache2::request::test_utils::with_request_rec;
    use crate::apache2::request::RequestContext;
    use crate::interface::telemetry::metrics::test_utils::with_mock_zero_metrics;
    use crate::schema::tile::config::ModuleConfig;

    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn test_not_handled() -> Result<(), Box<dyn Error>> {
        let mut layer_handler = DescriptionHandler { };
        let layer_name = String::from("default");
        let tile_config = ModuleConfig::new();
        let layer_config = tile_config.layers.get(&layer_name).unwrap();
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
                request.uri = uri.into_raw();
                let handle_context = HandleContext {
                    request_context: RequestContext::create_with_tile_config(request, &tile_config)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let request = request::Request {
                    header: request::Header::new_with_layer(
                        handle_context.request_context.record,
                        handle_context.request_context.connection.record,
                        handle_context.request_context.get_host().record,
                        &layer_name,
                    ),
                    body: request::BodyVariant::ReportStatistics,
                };

                assert!(layer_handler.handle(&handle_context, &request).result?.is_not_handled(), "Expected to not handle");
                Ok(())
            })
        })
    }

    #[test]
    fn test_default_config_json() -> Result<(), Box<dyn Error>> {
        let mut layer_handler = DescriptionHandler { };
        let layer_name = String::from("default");
        let tile_config = ModuleConfig::new();
        let layer_config = tile_config.layers.get(&layer_name).unwrap();
        with_request_rec(|request| {
            with_mock_zero_metrics(|cache_metrics, render_metrics, response_metrics| {
                let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
                request.uri = uri.into_raw();
                let handle_context = HandleContext {
                    request_context: RequestContext::create_with_tile_config(request, &tile_config)?,
                    cache_metrics,
                    render_metrics,
                    response_metrics,
                };
                let request = request::Request {
                    header: request::Header::new_with_layer(
                        handle_context.request_context.record,
                        handle_context.request_context.connection.record,
                        handle_context.request_context.get_host().record,
                        &layer_name,
                    ),
                    body: request::BodyVariant::DescribeLayer,
                };

                let actual_response = layer_handler.handle(&handle_context, &request).result?.expect_handled();
                let expected_data = response::Description {
                    tilejson: "2.0.0",
                    schema: "xyz",
                    name: layer_name.clone(),
                    description: layer_config.description.clone(),
                    attribution: layer_config.attribution.clone(),
                    minzoom: layer_config.min_zoom,
                    maxzoom: layer_config.max_zoom,
                    tiles: vec![String::from("http://localhost/osm/{z}/{x}/{y}.png")],
                };
                let expected_response = response::Response {
                    header: response::Header::new(
                        handle_context.request_context.record,
                        handle_context.request_context.connection.record,
                        handle_context.request_context.get_host().record,
                        &mime::APPLICATION_JSON,
                    ),
                    body: response::BodyVariant::Description(expected_data),
                };
                assert_eq!(expected_response, actual_response, "Incorrect handling");
                Ok(())
            })
        })
    }
}
