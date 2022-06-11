use fixedstr::fstr;

use std::mem::size_of;


pub type LayerName = fstr<16>;

/// TODO: replace when const fn are allowed in traits
pub const fn max_layer_name_char_len() -> usize {
    let alpha_char_len = 'A'.len_utf8();
    let max_layer_len = size_of::<LayerName>();
    return max_layer_len / alpha_char_len;

}

pub struct TileIdentity {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub layer: LayerName,
}
