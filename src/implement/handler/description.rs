use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::identity::LayerName;
use crate::interface::handler::{
    HandleContext, IOContext, RequestHandler,
};

use chrono::Utc;
use mime;

use std::any::type_name;


pub struct DescriptionHandlerState { }

impl DescriptionHandlerState {
    pub fn new(_config: &ModuleConfig) -> Result<DescriptionHandlerState, InvalidConfigError> {
        Ok(
            DescriptionHandlerState {  }
        )
    }
}

impl RequestHandler for DescriptionHandlerState {
    fn handle(
        &mut self,
        context: &HandleContext,
        _io: &mut IOContext,
        request: &request::SlippyRequest,
    ) -> HandleOutcome {
        let before_timestamp = Utc::now();
        let layer = match request.body {
            request::BodyVariant::DescribeLayer => &request.header.layer,
            _ => {
                return HandleOutcome::Ignored;
            },
        };
        let description = describe(context.module_config, layer);
        let response = response::SlippyResponse {
            header: response::Header::new(
                context.request.record,
                &mime::APPLICATION_JSON,
            ),
            body: response::BodyVariant::Description(description),
        };
        let after_timestamp = Utc::now();
        return HandleOutcome::Processed(
            HandleRequestResult {
                before_timestamp,
                after_timestamp,
                result: Ok(response),
            }
        );
    }

    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}

fn describe(config: &ModuleConfig, layer: &LayerName) -> response::Description {
    let layer_config = &config.layers[layer];
    let mut value = response::Description {
        tilejson: "2.0.0",
        schema: "xyz",
        name: String::from(layer_config.name.as_str()),
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
    use crate::schema::apache2::config::ModuleConfig;
    use crate::schema::apache2::connection::Connection;
    use crate::schema::apache2::request::Apache2Request;
    use crate::schema::apache2::virtual_host::VirtualHost;
    use crate::interface::apache2::PoolStored;
    use crate::interface::communication::test_utils::EmptyResultCommunicationInventory;
    use crate::interface::storage::test_utils::BlankStorageInventory;
    use crate::interface::telemetry::test_utils::NoOpZeroTelemetryInventory;
    use crate::framework::apache2::record::test_utils::with_request_rec;

    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn test_not_handled() -> Result<(), Box<dyn Error>> {
        let module_config = ModuleConfig::new();
        let mut description_state = DescriptionHandlerState::new(&module_config)?;
        let layer_name = LayerName::from("default");
        let layer_config = module_config.layers.get(&layer_name).unwrap();
        let telemetry = NoOpZeroTelemetryInventory::new();
        let mut communication = EmptyResultCommunicationInventory::new();
        let mut storage = BlankStorageInventory::new();
        with_request_rec(|request| {
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            request.uri = uri.into_raw();
            let handle_context = HandleContext::new(
                request,
                &module_config,
                &telemetry,
            );
            let mut io_context = IOContext::new(
                &mut communication,
                &mut storage,
            );
            let request = request::SlippyRequest {
                header: request::Header::new_with_layer(
                    handle_context.request.record,
                    &layer_name,
                ),
                body: request::BodyVariant::ReportStatistics,
            };

            assert!(description_state.handle(&handle_context, &mut io_context, &request).is_ignored(), "Expected to not handle");
            Ok(())
        })
    }

    #[test]
    fn test_default_config_json() -> Result<(), Box<dyn Error>> {
        let module_config = ModuleConfig::new();
        let mut description_state = DescriptionHandlerState::new(&module_config)?;
        let layer_name = LayerName::from("default");
        let layer_config = module_config.layers.get(&layer_name).unwrap();
        let telemetry = NoOpZeroTelemetryInventory::new();
        let mut communication = EmptyResultCommunicationInventory::new();
        let mut storage = BlankStorageInventory::new();
        with_request_rec(|request| {
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            request.uri = uri.into_raw();
            let handle_context = HandleContext {
                module_config: &module_config,
                host: VirtualHost::find_or_allocate_new(request)?,
                request: Apache2Request::create_with_tile_config(request)?,
                telemetry: &telemetry,
            };
            let mut io_context = IOContext::new(
                &mut communication,
                &mut storage,
            );
            let request = request::SlippyRequest {
                header: request::Header::new_with_layer(
                    handle_context.request.record,
                    &layer_name,
                ),
                body: request::BodyVariant::DescribeLayer,
            };

            let actual_response = description_state.handle(&handle_context, &mut io_context, &request).expect_processed().result?;
            let expected_data = response::Description {
                tilejson: "2.0.0",
                schema: "xyz",
                name: String::from(layer_name.as_str()),
                description: layer_config.description.clone(),
                attribution: layer_config.attribution.clone(),
                minzoom: layer_config.min_zoom,
                maxzoom: layer_config.max_zoom,
                tiles: vec![String::from("http://localhost/osm/{z}/{x}/{y}.png")],
            };
            let expected_response = response::SlippyResponse {
                header: response::Header::new(
                    handle_context.request.record,
                    &mime::APPLICATION_JSON,
                ),
                body: response::BodyVariant::Description(expected_data),
            };
            assert_eq!(expected_response, actual_response, "Incorrect handling");
            Ok(())
        })
    }
}
