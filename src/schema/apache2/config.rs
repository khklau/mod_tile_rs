use crate::schema::tile::identity::LayerName;

use std::collections::hash_map::HashMap;
use std::time::Duration;


#[derive(Debug)]
pub struct ModuleConfig {
    pub renderd: RenderdConfig,
    pub layers: HashMap<LayerName, LayerConfig>,
}

impl ModuleConfig {
    pub fn new() -> ModuleConfig {
        let mut value = ModuleConfig {
            renderd: RenderdConfig::new(),
            layers: HashMap::new(),
        };
        value.layers.insert(LayerName::from("default"), LayerConfig::new());
        value
    }
}

#[derive(Debug)]
pub struct RenderdConfig {
    pub store_uri: String,
    pub ipc_uri: String,
    pub render_timeout: Duration,
}

impl RenderdConfig {
    pub fn new() -> RenderdConfig {
        RenderdConfig {
            store_uri: String::from("/var/cache/renderd"),
            ipc_uri: String::from("/var/run/renderd/renderd.sock"),
            render_timeout: Duration::new(0, 0),
        }
    }
}

pub const MAX_ZOOM_SERVER: usize = 30;

#[derive(Debug)]
pub struct LayerConfig {
    pub name: LayerName,
    pub base_url: String,
    pub description: String,
    pub attribution: String,
    pub min_zoom: u64,
    pub max_zoom: u64,
    pub file_extension: String,
    pub mime_type: String,
    pub host_name: String,
    pub parameters_allowed: bool,
}

impl LayerConfig {
    pub fn new() -> LayerConfig {
        let mut config = LayerConfig {
            name: LayerName::from("default"),
            base_url: String::from("/osm"),
            description: String::from("default"),
            attribution: String::from("default"),
            min_zoom: 0,
            max_zoom: 20,
            file_extension: String::from("png"),
            mime_type: String::from("image/png"),
            host_name: String::new(),
            parameters_allowed: false,
        };
        config.set_host_name("localhost");
        config
    }

    pub fn set_host_name(
        &mut self,
        host_name: &str
    ) -> () {
        self.host_name = format!("http://{}", host_name);
    }
}
