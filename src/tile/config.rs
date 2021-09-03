use configparser::ini::Ini;

use std::error::Error;
use std::fmt;
use std::path::Path;
use std::result::Result;
use std::string::String;

pub struct TileConfig {
    pub name: String,
    pub description: String,
    pub attribution: String,
    pub min_zoom: u64,
    pub max_zoom: u64,
    pub store_uri: String,
    pub base_url: String,
    pub ipc_uri: String,
    pub file_extension: String,
    pub parameters_allowed: bool,
}

impl TileConfig {
    pub fn new() -> TileConfig {
        TileConfig {
            name: String::from("default"),
            description: String::from("default"),
            attribution: String::from("default"),
            min_zoom: 0,
            max_zoom: 20,
            store_uri: String::from("/var/cache/renderd"),
            base_url: String::from("/osm"),
            ipc_uri: String::from("/var/run/renderd/renderd.sock"),
            file_extension: String::from("png"),
            parameters_allowed: false,
        }
    }

    pub fn load(path: &Path) -> Result<Self, ParseError> {
        let mut ini = Ini::new();
        ini.load(path)?;
        return _parse(&ini);
    }
}

fn _parse(ini: &Ini) -> Result<TileConfig, ParseError> {
    let mut config = TileConfig::new();
    for section_name in &(ini.sections()) {
        config.name = match section_name.to_lowercase().as_str() {
            "renderd" | "mapnik" => config.name,
            _ => section_name.to_string(),
        };
        if let Some(description) = ini.get(section_name.as_str(), "description") {
            config.description = description;
        }
        if let Some(attribution) = ini.get(section_name.as_str(), "attribution") {
            config.attribution = attribution;
        }
        if let Some(min_zoom) = ini.getuint(section_name.as_str(), "minzoom")? {
            config.min_zoom = min_zoom;
        }
        if let Some(max_zoom) = ini.getuint(section_name.as_str(), "maxzoom")? {
            config.max_zoom = max_zoom;
        }
        if let Some(tile_dir) = ini.get(section_name.as_str(), "tile_dir") {
            config.store_uri = tile_dir;
        }
        if let Some(uri) = ini.get(section_name.as_str(), "uri") {
            config.base_url = uri.trim_end_matches("/").to_string();
        }
        if let Some(socket_name) = ini.get(section_name.as_str(), "socketname") {
            config.ipc_uri = socket_name;
        }
        if let Some(parameters_allowed) = ini.getbool(section_name.as_str(), "parameterize_style")? {
            config.parameters_allowed = parameters_allowed;
        }
    }
    return Ok(config);
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

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TileConfig parsing failed: {}", self.reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::boxed::Box;
    use std::error::Error;
    use std::path::PathBuf;

    #[test]
    fn test_load_basic_valid_file() -> Result<(), ParseError> {
        let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("resources/test/tile/basic_valid.conf");
        let actual_config = TileConfig::load(file_path.as_path())?;
        assert_eq!(String::from("basic"), actual_config.name, "Failed to load name");
        assert_eq!(String::from("/var/cache/test"), actual_config.store_uri, "Failed to load store_uri");
        assert_eq!(String::from("/test"), actual_config.base_url, "Failed to load base_url");
        assert_eq!(String::from("/var/run/test.sock"), actual_config.ipc_uri, "Failed to load ipc_uri");
        assert_eq!(String::from("png"), actual_config.file_extension, "Failed to load file_extension");
        Ok(())
    }

    #[test]
    fn test_load_basic_invalid_file() -> Result<(), Box<dyn Error>> {
        let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("resources/test/tile/basic_invalid.conf");
        assert!(TileConfig::load(file_path.as_path()).is_ok(), "Invalid file was parsed");
        Ok(())
    }

    #[test]
    fn test_parse_config_name() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.set("renderd", "socketname", Some(String::from("/var/run/renderd/renderd.sock")));
        ini.set("MAPNIK", "font_dir", Some(String::from("/usr/share/fonts/")));
        ini.set("foobar", "uri", Some(String::from("/foo/")));
        let actual_config = _parse(&ini)?;
        assert_eq!("foobar", actual_config.name, "Failed to parse config name from section");
        assert_eq!("/var/run/renderd/renderd.sock", actual_config.ipc_uri, "Failed to parse socketname");
        Ok(())
    }

    #[test]
    fn test_parse_uri_with_trailing_slash() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.set("basic", "uri", Some(String::from("/foo/")));
        let actual_config = _parse(&ini)?;
        assert_eq!("/foo", actual_config.base_url, "Failed to trim trailing slash from URI");
        Ok(())
    }

    #[test]
    fn test_parse_parameterize_style_as_bool() -> Result<(), Box<dyn Error>> {
        let mut ini1 = Ini::new();
        ini1.set("basic", "parameterize_style", Some(String::from("TRUE")));
        let actual_config1 = _parse(&ini1)?;
        assert!(actual_config1.parameters_allowed, "Failed to parse parameterize_style");

        let mut ini2 = Ini::new();
        ini2.set("basic", "parameterize_style", Some(String::from("false")));
        let actual_config2 = _parse(&ini2)?;
        assert!(!actual_config2.parameters_allowed, "Failed to parse parameterize_style");
        Ok(())
    }

    #[test]
    fn test_parse_invalid_parameterize_style() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.set("basic", "parameterize_style", Some(String::from("yes")));
        assert!(_parse(&ini).is_err(), "Invalid parameterize_size value was not rejected");
        Ok(())
    }

    #[test]
    fn test_parse_uppercase_section_and_key() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.set("BASIC", "TILE_DIR", Some(String::from("/var/cache/renderd/")));
        let actual_config = _parse(&ini)?;
        assert_eq!("/var/cache/renderd/", actual_config.store_uri, "Failed to parse upper case tile_dir");
        Ok(())
    }
}
