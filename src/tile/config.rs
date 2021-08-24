use configparser::ini::Ini;

use std::path::Path;
use std::result::Result;
use std::string::String;

pub struct Config {
    name: String,
    store_uri: String,
    base_url: String,
    ipc_uri: String,
    file_extension: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            name: String::from("default"),
            store_uri: String::from("/var/cache/renderd"),
            base_url: String::from("/osm/"),
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

pub fn parse(path: &Path) -> Result<Config, ParseError> {
    let mut ini = Ini::new();
    let mut config = Config::new();
    ini.load(path)?;
    for section_name in &(ini.sections()) {
        config.name = match section_name.to_lowercase().as_str() {
            "renderd" | "mapnik" => config.name,
            _ => section_name.to_string(),
        };
        if let Some(tile_dir) = ini.get(section_name.as_str(), "tile_dir") {
            config.store_uri = tile_dir;
        }
        if let Some(uri) = ini.get(section_name.as_str(), "uri") {
            config.base_url = uri;
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
    fn test_parse_basic_valid_file() -> Result<(), ParseError> {
        let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("resources/test/tile/basic_valid.conf");
        let actual_config = parse(file_path.as_path())?;
        assert_eq!(String::from("basic"), actual_config.name, "Failed to parse name");
        assert_eq!(String::from("/var/cache/test"), actual_config.store_uri, "Failed to parse store_uri");
        assert_eq!(String::from("/test/"), actual_config.base_url, "Failed to parse base_url");
        assert_eq!(String::from("/var/run/test.sock"), actual_config.ipc_uri, "Failed to parse ipc_uri");
        assert_eq!(String::from("png"), actual_config.file_extension, "Failed to parse file_extension");
        Ok(())
    }
}
