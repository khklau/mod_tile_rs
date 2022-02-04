use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum TileSource {
    Cache,
    Render,
}
