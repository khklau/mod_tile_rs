use crate::binding::renderd_protocol::{protocol, protoCmd};
use crate::schema::apache2::config::RenderdConfig;
use crate::schema::renderd::error::RenderError;
use crate::schema::renderd::request::{RenderRequestCommand, RenderRequestVersion};
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::tile::identity::TileIdentity;
use crate::schema::tile::tile_ref::TileRef;
use crate::io::interface::IOContext;


pub fn create_request(
    _config: &RenderdConfig,
    _slippy: &SlippyRequest,
) -> protocol {
    let result = protocol {
        ver: RenderRequestVersion::Three as std::os::raw::c_int,
        cmd: RenderRequestCommand::Render as protoCmd,
        x: 0 as std::os::raw::c_int,
        y: 0 as std::os::raw::c_int,
        z: 0 as std::os::raw::c_int,
        xmlname: [0; 41usize],
        mimetype: [0; 41usize],
        options: [0; 41usize],
    };
    result
}

pub trait TileRenderer {
    fn render_tile(
        &mut self,
        io: &mut IOContext,
        tile_id: TileIdentity,
        request: &protocol,
        response: &mut protocol,
        priority: u8,
    ) -> Result<TileRef, RenderError>;
}

pub trait RenderingInventory {
    fn tile_renderer(&mut self) -> &mut dyn TileRenderer;
}


#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::schema::http::encoding::ContentEncoding;
    use std::rc::Rc;
    use std::vec::Vec;

    pub struct MockTileRenderer {
        buffer: Rc<Vec<u8>>,
    }

    impl MockTileRenderer {
        pub fn new() -> MockTileRenderer {
            MockTileRenderer {
                buffer: Rc::new(Vec::new()),
            }
        }
    }

    impl TileRenderer for MockTileRenderer {
        fn render_tile(
            &mut self,
            io: &mut crate::io::interface::IOContext,
            tile_id: crate::schema::tile::identity::TileIdentity,
            request: &crate::binding::renderd_protocol::protocol,
            response: &mut crate::binding::renderd_protocol::protocol,
            priority: u8,
        ) -> Result<TileRef, crate::schema::renderd::error::RenderError> {
            Ok(
                TileRef {
                    raw_bytes: Rc::clone(&self.buffer),
                    begin: 0,
                    end: 1,
                    media_type: mime::IMAGE_PNG,
                    encoding: ContentEncoding::NotCompressed,
                }
            )
        }
    }

    pub struct NoOpRenderingInventory {
        tile_renderer: MockTileRenderer,
    }

    impl NoOpRenderingInventory {
        pub fn new() -> NoOpRenderingInventory {
            NoOpRenderingInventory {
                tile_renderer: MockTileRenderer::new(),
            }
        }
    }

    impl RenderingInventory for NoOpRenderingInventory {
        fn tile_renderer(&mut self) -> &mut dyn TileRenderer {
            &mut self.tile_renderer
        }
    }
}
