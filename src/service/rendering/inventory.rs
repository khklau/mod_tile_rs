use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::error::InvalidConfigError;
use crate::service::rendering::mapnik::Mapnik;
use crate::service::rendering::interface::{RenderingInventory, TileRenderer,};


pub struct RenderingState {
    mapnik: Mapnik,
}

impl RenderingState {
    pub fn new(config: &ModuleConfig) -> Result<RenderingState, InvalidConfigError> {
        Ok(
            RenderingState {
                mapnik: Mapnik::new(config)?,
            }
        )
    }
}

impl RenderingInventory for RenderingState {
    fn tile_renderer(&mut self) -> &mut dyn TileRenderer {
        &mut self.mapnik
    }
}


mod tests {

    use super::*;
    use std::boxed::Box;
    use std::error::Error;

    #[test]
    fn test_new() -> Result<(), Box<dyn Error>> {
        let module_config = ModuleConfig::new();
        let _value = RenderingState::new(&module_config)?;
        return Ok(())
    }
}

