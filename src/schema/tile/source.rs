use serde::Serialize;
use variant_count::VariantCount;

use std::marker::Copy;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, VariantCount)]
pub enum TileSource {
    Render = 0,
    Cache,
}
