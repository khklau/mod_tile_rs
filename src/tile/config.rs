use configparser::ini::Ini;

use std::path::Path;
use std::result::Result;
use std::string::String;

pub struct TileConfig {
    pub name: String,
    pub store_uri: String,
    pub base_url: String,
    pub ipc_uri: String,
    pub file_extension: String,
}

impl TileConfig {
    pub fn new() -> TileConfig {
        TileConfig {
            name: String::from("default"),
            store_uri: String::from("/var/cache/renderd"),
            base_url: String::from("/osm"),
            ipc_uri: String::from("/var/run/renderd/renderd.sock"),
            file_extension: String::from("png")
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    reason: String,
}

impl From<String> for ParseError {
    fn from(reason: String) -> Self {
        return ParseError { reason };
    }
}

pub fn load(path: &Path) -> Result<TileConfig, ParseError> {
    let mut ini = Ini::new();
    ini.load(path)?;
    return _parse(&ini);
}

fn _parse(ini: &Ini) -> Result<TileConfig, ParseError> {
    let mut config = TileConfig::new();
    for section_name in &(ini.sections()) {
        config.name = match section_name.to_lowercase().as_str() {
            "renderd" | "mapnik" => config.name,
            _ => section_name.to_string(),
        };
        if let Some(tile_dir) = ini.get(section_name.as_str(), "tile_dir") {
            config.store_uri = tile_dir;
        }
        if let Some(uri) = ini.get(section_name.as_str(), "uri") {
            config.base_url = uri.trim_end_matches("/").to_string();
        }
        if let Some(socket_name) = ini.get(section_name.as_str(), "socketname") {
            config.ipc_uri = socket_name;
        }
    }
    return Ok(config);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_basic_valid_file() -> Result<(), ParseError> {
        let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("resources/test/tile/basic_valid.conf");
        let actual_config = load(file_path.as_path())?;
        assert_eq!(String::from("basic"), actual_config.name, "Failed to load name");
        assert_eq!(String::from("/var/cache/test"), actual_config.store_uri, "Failed to load store_uri");
        assert_eq!(String::from("/test"), actual_config.base_url, "Failed to load base_url");
        assert_eq!(String::from("/var/run/test.sock"), actual_config.ipc_uri, "Failed to load ipc_uri");
        assert_eq!(String::from("png"), actual_config.file_extension, "Failed to load file_extension");
        Ok(())
    }
}
