use crate::handler::interface::{ HandleOutcome, HandleRequestResult, RequestHandler };

use crate::apache2::request::RequestContext;
use crate::slippy::request;
use crate::slippy::response;
use crate::tile::config::TileConfig;

use mime;

use std::string::String;


pub struct DescriptionHandler { }
impl RequestHandler for DescriptionHandler {
    fn handle(
        &mut self,
        context: &RequestContext,
        request: &request::Request,
    ) -> HandleRequestResult {
        let layer = match request.body {
            request::BodyVariant::DescribeLayer => &request.header.layer,
            _ => {
                return Ok(HandleOutcome::NotHandled);
            },
        };
        let description = describe(context.get_config(), layer);
        let response = response::Response {
            header: response::Header::new(
                context.record,
                context.connection.record,
                context.get_host().record,
                &mime::APPLICATION_JSON,
            ),
            body: response::BodyVariant::Description(description),
        };
        Ok(HandleOutcome::Handled(response))
    }
}

fn describe(config: &TileConfig, layer: &String) -> response::Description {
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
    use crate::tile::config::TileConfig;

    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn test_not_handled() -> Result<(), Box<dyn Error>> {
        let mut layer_handler = DescriptionHandler { };
        let layer_name = String::from("default");
        let tile_config = TileConfig::new();
        let layer_config = tile_config.layers.get(&layer_name).unwrap();
        with_request_rec(|record| {
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let request = request::Request {
                header: request::Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &layer_name,
                ),
                body: request::BodyVariant::ReportStatistics,
            };

            assert!(layer_handler.handle(context, &request)?.is_not_handled(), "Expected to not handle");
            Ok(())
        })
    }

    #[test]
    fn test_default_config_json() -> Result<(), Box<dyn Error>> {
        let mut layer_handler = DescriptionHandler { };
        let layer_name = String::from("default");
        let tile_config = TileConfig::new();
        let layer_config = tile_config.layers.get(&layer_name).unwrap();
        with_request_rec(|record| {
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            record.uri = uri.into_raw();
            let context = RequestContext::create_with_tile_config(record, &tile_config)?;
            let request = request::Request {
                header: request::Header::new_with_layer(
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &layer_name,
                ),
                body: request::BodyVariant::DescribeLayer,
            };

            let actual_response = layer_handler.handle(context, &request)?.expect_handled();
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
                    context.record,
                    context.connection.record,
                    context.get_host().record,
                    &mime::APPLICATION_JSON,
                ),
                body: response::BodyVariant::Description(expected_data),
            };
            assert_eq!(expected_response, actual_response, "Incorrect handling");
            Ok(())
        })
    }
}
