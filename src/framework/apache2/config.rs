use crate::schema::apache2::config::{ ModuleConfig, RenderdConfig, LayerConfig };
use crate::schema::tile::identity::{ LayerName, max_layer_name_char_len };

use configparser::ini::Ini;

use std::error::Error;
use std::fmt;
use std::option::Option;
use std::path::Path;
use std::result::Result;
use std::string::String;


pub trait Loadable {
    fn load(
        path: &Path,
        server_name: Option<&str>,
    ) -> Result<ModuleConfig, ParseError>;
}

impl Loadable for ModuleConfig {
    fn load(
        path: &Path,
        server_name: Option<&str>,
    ) -> Result<ModuleConfig, ParseError> {
        let mut ini = Ini::new();
        ini.load(path)?;
        return parse(&ini, server_name);
    }
}

fn parse(
    ini: &Ini,
    server_name: Option<&str>,
) -> Result<ModuleConfig, ParseError> {
    let mut config = ModuleConfig::new();
    'sections: for section_name in &(ini.sections()) {
        match section_name.to_lowercase().as_str() {
            "mapnik" => {
                continue 'sections;
            },
            "renderd" => {
                config.renderd = parse_renderd(ini, section_name)?;
            },
            _ => {
                let layer_name = match LayerName::try_make(section_name.as_str()) {
                    Ok(name) => name,
                    Err(_) => {
                        return Err(
                            ParseError{
                                reason: format!(
                                    "Layer name {} exceeds length limit of {}",
                                    section_name,
                                    max_layer_name_char_len(),
                                ),
                            }
                        );
                    },
                };
                let layer = parse_layer(ini, &layer_name, server_name)?;
                config.layers.insert(layer_name, layer);
            },
        };
    }
    return Ok(config);
}

fn parse_renderd(ini: &Ini, section_name: &String) -> Result<RenderdConfig, ParseError> {
    let mut config = RenderdConfig::new();
    if let Some(tile_dir) = ini.get(section_name.as_str(), "tile_dir") {
        config.store_uri = tile_dir;
    }
    if let Some(socket_name) = ini.get(section_name.as_str(), "socketname") {
        config.ipc_uri = socket_name;
    }
    return Ok(config);
}

fn parse_layer(
    ini: &Ini,
    section_name: &LayerName,
    server_name: Option<&str>,
) -> Result<LayerConfig, ParseError> {
    let mut config = LayerConfig::new();
    config.name = section_name.clone();
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
    if let Some(uri) = ini.get(section_name.as_str(), "uri") {
        config.base_url = uri.trim_end_matches("/").to_string();
    }
    if let Some(parameters_allowed) = ini.getbool(section_name.as_str(), "parameterize_style")? {
        config.parameters_allowed = parameters_allowed;
    }
    if let Some(alias) = ini.get(section_name.as_str(), "server_alias") {
        config.set_host_name(alias.as_str());
    } else if let Some(name) = server_name {
        config.set_host_name(name);
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
        let actual_config = ModuleConfig::load(file_path.as_path(), None)?;
        let layer = LayerName::from("basic");
        assert_eq!(
            "basic",
            actual_config.layers.get(&layer).unwrap().name,
            "Failed to load name"
        );
        assert_eq!(
            "/test",
            actual_config.layers.get(&layer).unwrap().base_url,
            "Failed to load base_url"
        );
        assert_eq!(
            "png",
            actual_config.layers.get(&layer).unwrap().file_extension,
            "Failed to load file_extension"
        );
        assert_eq!(
            "http://localhost",
            actual_config.layers.get(&layer).unwrap().host_name,
            "Failed to load default hostname"
        );
        assert_eq!(
            "/var/cache/test",
            actual_config.renderd.store_uri,
            "Failed to load store_uri"
        );
        assert_eq!(
            "/var/run/test.sock",
            actual_config.renderd.ipc_uri,
            "Failed to load ipc_uri"
        );
        Ok(())
    }

    #[test]
    fn test_load_basic_invalid_file() -> Result<(), Box<dyn Error>> {
        let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("resources/test/tile/basic_invalid.conf");
        assert!(ModuleConfig::load(file_path.as_path(), None).is_ok(), "Invalid file was parsed");
        Ok(())
    }

    #[test]
    fn test_parse_config_name() -> Result<(), Box<dyn Error>> {
        let layer = LayerName::from("foobar");
        let mut ini = Ini::new();
        ini.set("renderd", "socketname", Some(String::from("/var/run/renderd/renderd.sock")));
        ini.set("MAPNIK", "font_dir", Some(String::from("/usr/share/fonts/")));
        ini.set(layer.as_str(), "uri", Some(String::from("/foo/")));
        let actual_config = parse(&ini, None)?;
        assert_eq!(
            layer,
            actual_config.layers.get(&layer).unwrap().name,
            "Failed to parse config name from section"
        );
        Ok(())
    }

    #[test]
    fn test_parse_renderd_config() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.set("renderd", "socketname", Some(String::from("/var/run/renderd/renderd.sock")));
        ini.set("RENDERD", "TILE_DIR", Some(String::from("/var/cache/renderd/")));
        let actual_config = parse(&ini, None)?;
        assert_eq!("/var/run/renderd/renderd.sock", actual_config.renderd.ipc_uri, "Failed to parse socketname");
        assert_eq!("/var/cache/renderd/", actual_config.renderd.store_uri, "Failed to parse tile_dir");
        Ok(())
    }

    #[test]
    fn test_parse_uri_with_trailing_slash() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        let layer = LayerName::from("basic");
        ini.set(layer.as_str(), "uri", Some(String::from("/foo/")));
        let actual_config = parse(&ini, None)?;
        assert_eq!(
            "/foo",
            actual_config.layers.get(&layer).unwrap().base_url,
            "Failed to trim trailing slash from URI");
        Ok(())
    }

    #[test]
    fn test_parse_parameterize_style_as_bool() -> Result<(), Box<dyn Error>> {
        let layer1 = LayerName::from("basic");
        let mut ini1 = Ini::new();
        ini1.set(layer1.as_str(), "parameterize_style", Some(String::from("TRUE")));
        let actual_config1 = parse(&ini1, None)?;
        assert!(
            actual_config1.layers.get(&layer1).unwrap().parameters_allowed,
            "Failed to parse parameterize_style");

        let layer2 = LayerName::from("directory");
        let mut ini2 = Ini::new();
        ini2.set(layer2.as_str(), "parameterize_style", Some(String::from("false")));
        let actual_config2 = parse(&ini2, None)?;
        assert!(
            !actual_config2.layers.get(&layer2).unwrap().parameters_allowed,
            "Failed to parse parameterize_style");
        Ok(())
    }

    #[test]
    fn test_parse_server_alias() -> Result<(), Box<dyn Error>> {
        let layer1 = LayerName::from("basic");
        let mut ini1 = Ini::new();
        ini1.set(layer1.as_str(), "server_alias", Some(String::from("webserver")));
        let actual_config1 = parse(&ini1, None)?;
        assert_eq!(
            "http://webserver",
            actual_config1.layers.get(&layer1).unwrap().host_name,
            "Failed to parse server_alias");

        let layer2 = LayerName::from("directory");
        let mut ini2 = Ini::new();
        ini2.set(layer2.as_str(), "uri", Some(String::from("/foo/")));
        let actual_config2 = parse(&ini2, Some("myserver"))?;
        assert_eq!(
            "http://myserver",
            actual_config2.layers.get(&layer2).unwrap().host_name,
            "Failed to use server name as hostname when server_alias is not specified");

        let layer3 = LayerName::from("custom");
        let mut ini3 = Ini::new();
        ini3.set(layer3.to_str(), "uri", Some(String::from("/bar/")));
        let actual_config3 = parse(&ini3, None)?;
        assert_eq!(
            "http://localhost",
            actual_config3.layers.get(&layer3).unwrap().host_name,
            "Failed to use default hostname when both server_alias and server name are not specified");
        Ok(())
    }

    #[test]
    fn test_parse_invalid_parameterize_style() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.set("basic", "parameterize_style", Some(String::from("yes")));
        assert!(parse(&ini, None).is_err(), "Invalid parameterize_size value was not rejected");
        Ok(())
    }

    #[test]
    fn test_parse_uppercase_section_and_key() -> Result<(), Box<dyn Error>> {
        let mut ini = Ini::new();
        ini.set("RENDERD", "TILE_DIR", Some(String::from("/var/cache/renderd/")));
        let actual_config = parse(&ini, None)?;
        assert_eq!("/var/cache/renderd/", actual_config.renderd.store_uri, "Failed to parse upper case tile_dir");
        Ok(())
    }
}
