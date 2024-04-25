use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::request;
use crate::schema::slippy::response;
use crate::schema::tile::identity::LayerName;
use crate::framework::apache2::context::HostContext;

use chrono::Utc;
use mime;

use std::any::type_name;


pub struct DescriptionContext<'c> {
    pub host: HostContext<'c>,
}

impl<'c> DescriptionContext<'c> {
    pub fn module_config(&self) -> &'c ModuleConfig {
        self.host.module_config
    }

    pub fn host(&self) -> &'c VirtualHost<'c> {
        self.host.host
    }
}


pub struct DescriptionHandlerState { }


impl DescriptionHandlerState {
    pub fn new(_config: &ModuleConfig) -> Result<DescriptionHandlerState, InvalidConfigError> {
        Ok(
            DescriptionHandlerState {  }
        )
    }

    pub fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    pub fn describe_layer(
        &mut self,
        context: &DescriptionContext,
        header: &request::Header,
    ) -> HandleRequestResult {
        let before_timestamp = Utc::now();
        let layer = &header.layer;
        let description = describe(context.module_config(), layer);
        let response = response::SlippyResponse {
            header: response::Header {
                mime_type: mime::APPLICATION_JSON.clone(),
            },
            body: response::BodyVariant::Description(description),
        };
        let after_timestamp = Utc::now();
        return HandleRequestResult {
            before_timestamp,
            after_timestamp,
            result: Ok(response),
        };
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
    use crate::core::identifier::generate_id;
    use crate::io::communication::interface::test_utils::EmptyResultCommunicationInventory;
    use crate::io::storage::interface::test_utils::BlankStorageInventory;
    use crate::service::telemetry::interface::test_utils::NoOpZeroTelemetryInventory;
    use crate::framework::apache2::record::test_utils::with_request_rec;

    use chrono::Utc;
    use std::error::Error as StdError;
    use std::ffi::CString;

    #[test]
    fn test_describe_layer_with_default_config() -> Result<(), Box<dyn StdError>> {
        let module_config = ModuleConfig::new();
        let mut description_state = DescriptionHandlerState::new(&module_config)?;
        let layer_name = LayerName::from("default");
        let layer_config = module_config.layers.get(&layer_name).unwrap();
        let telemetry = NoOpZeroTelemetryInventory::new();
        let mut communication = EmptyResultCommunicationInventory::new();
        let mut storage = BlankStorageInventory::new();
        with_request_rec(|record| {
            let uri = CString::new(format!("{}/tile-layer.json", layer_config.base_url))?;
            record.uri = uri.clone().into_raw();
            let context = DescriptionContext {
                host: HostContext::new(&module_config, record),
            };
            let header = request::Header {
                layer: layer_name.clone(),
                request_id: generate_id(),
                uri: uri.into_string()?,
                received_timestamp: Utc::now(),
            };
            let actual_response = description_state.describe_layer(
                &context,
                &header
            ).result?;
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
                header: response::Header {
                    mime_type: mime::APPLICATION_JSON.clone(),
                },
                body: response::BodyVariant::Description(expected_data),
            };
            assert_eq!(expected_response, actual_response, "Incorrect handling");
            Ok(())
        })
    }
}
