use crate::schema::apache2::config::RenderdConfig;
use crate::schema::tile::identity::LayerName;

use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;


pub fn data_import_completion_time(
    config: &RenderdConfig,
    layer_name: &LayerName,
) -> Option<SystemTime> {
    let mut path = PathBuf::new();
    path.push(config.store_uri.as_str());
    path.push(layer_name.as_str());
    path.push(IMPORT_COMPLETE_FILE);
    File::open(path.as_path())
        .and_then(|file| { file.metadata() })
        .and_then(|metadata| { metadata.modified() })
        .ok()
}

const IMPORT_COMPLETE_FILE: &str = "planet-import-complete";
