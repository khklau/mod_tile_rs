use enum_iterator::IntoEnumIterator;
use serde::Serialize;

use std::marker::Copy;


#[derive(Copy, Clone, Debug, Eq, Hash, IntoEnumIterator, PartialEq, Serialize)]
pub enum TileAge {
    Fresh = 0,
    Old,
    VeryOld,
}
