use crate::sheet::SheetPosition;

pub mod world;
pub mod zone;

pub type TileId = String;

#[derive(Debug, Clone)]
pub struct TileAppearance {
    pub back: Option<SheetPosition>,
    pub fore: SheetPosition,
}
