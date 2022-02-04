use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum TileAge {
    Fresh,
    Old,
    VeryOld,
}
