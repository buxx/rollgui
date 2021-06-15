use crate::tile::{TileAppearance, TileId};
use coffee::graphics::{Image, Point, Rectangle, Sprite};
use std::collections::HashMap;

pub type SheetPosition = (i16, i16);

#[derive(Debug, Clone)]
pub struct TileSheet {
    image: Image,
    sources: HashMap<SheetPosition, Rectangle<u16>>,
    max_row_i: i16,
    max_col_i: i16,
    appearances: HashMap<TileId, TileAppearance>,
    tile_width: i16,
    tile_height: i16,
}

const APPEARANCES: [(&str, Option<SheetPosition>, SheetPosition); 95] = [
    ("SEA", None, (7, 0)),
    ("JUNGLE", None, (7, 4)),
    ("PLAIN", None, (7, 2)),
    ("HILL", None, (7, 5)),
    ("MOUNTAIN", None, (7, 6)),
    ("BEACH", None, (7, 1)),
    ("BACK_BEACH", None, (0, 3)),
    ("BACK_PLAIN", None, (0, 6)),
    ("BACK_JUNGLE", None, (0, 4)),
    ("BACK_HILL", None, (0, 4)),
    ("BACK_MOUNTAIN", None, (0, 5)),
    ("BACK_SEA", None, (0, 2)),
    ("UNKNOWN", None, (0, 0)),
    ("SAND", None, (0, 0)),
    ("DRY_BUSH", None, (2, 8)),
    ("ROCK", None, (2, 9)),
    ("SEA_WATER", None, (0, 2)),
    ("FRESH_WATER_TILE", None, (0, 1)),
    ("SHORT_GRASS", None, (1, 0)),
    ("HIGH_GRASS", None, (1, 3)),
    ("ROCKY_GROUND", None, (0, 0)),
    ("DIRT", None, (0, 0)),
    ("LEAF_TREE", None, (1, 7)),
    ("TROPICAL_TREE", None, (1, 8)),
    ("DEAD_TREE", None, (1, 9)),
    ("PLAYER", None, (6, 0)),
    ("PLAYER_LEFT", None, (6, 1)),
    ("CHARACTER", None, (6, 0)),
    ("STUFF_GENERIC", None, (3, 0)),
    ("BOTTLE", None, (3, 1)),
    ("BAG", None, (3, 2)),
    ("COAT", None, (3, 3)),
    ("ARMOR", None, (3, 4)),
    ("SHIELD", None, (3, 5)),
    ("WEAPON", None, (3, 6)),
    ("SPEAR", None, (3, 7)),
    ("CORPSE", None, (3, 11)),
    ("ANIMAL", None, (3, 9)),
    ("CRAFT", None, (3, 10)),
    ("RESOURCE_GENERIC", None, (5, 0)),
    ("COPPER_DEPOSIT", None, (0, 7)),
    ("TIN_DEPOSIT", None, (0, 8)),
    ("IRON_DEPOSIT", None, (0, 9)),
    ("FRESH_WATER", None, (5, 10)),
    ("SALTED_WATER", None, (5, 10)),
    ("BEACH_SAND", None, (5, 11)),
    ("SOIL", None, (5, 9)),
    ("WET_SOIL", None, (5, 9)),
    ("WOOD", None, (5, 6)),
    ("VEGETAL_FOOD_FRESH", None, (5, 3)),
    ("SHELLFISH_FRESH", None, (5, 1)),
    ("RAW_MEAT", None, (5, 4)),
    ("COOKED_MEAT", None, (5, 5)),
    ("SMOKED_MEAT", None, (5, 5)),
    ("ANIMAL_SKIN", None, (5, 8)),
    ("GRAMINEAE", None, (5, 13)),
    ("BREAD", None, (5, 14)),
    ("RAW_STONE", None, (5, 12)),
    ("LEATHER_PIECE", None, (5, 7)),
    ("BUILD_GENERIC", None, (4, 1)),
    ("CAMPFIRE", None, (4, 1)),
    ("WALL", None, (4, 2)),
    ("WOOD_FENCE", None, (4, 2)),
    ("STONE_WALL", None, (4, 3)),
    ("LOOM", None, (4, 6)),
    ("BRUSHWOOD_EDGE", None, (4, 4)),
    ("SOIL_WALL", None, (4, 5)),
    ("BASKETRY_BAG", None, (3, 12)),
    ("SKIN_BAG", None, (3, 14)),
    ("LEATHER_BAG", None, (3, 13)),
    ("TRAVOIS", None, (3, 20)),
    ("CLOTH_BAG", None, (3, 15)),
    ("ANIMAL_SKIN_CLOTHES", None, (3, 17)),
    ("LEATHER_CLOTHES", None, (3, 16)),
    ("LEATHER_BRIGANDINE", None, (3, 19)),
    ("HARE", None, (3, 26)),
    ("PIG", None, (3, 25)),
    ("GOAT", None, (3, 24)),
    ("MOORHEN", None, (3, 23)),
    ("CRAB", None, (3, 22)),
    ("RAW_BRICK", None, (3, 27)),
    ("FIRED_BRICK", None, (3, 28)),
    ("RAW_BRICK_WALL", None, (4, 7)),
    ("FIRED_BRICK_WALL", None, (4, 8)),
    ("SOIL_KILN", None, (4, 9)),
    ("RAW_BRICK_KILN", None, (4, 10)),
    ("FIRED_BRICK_KILN", None, (4, 11)),
    ("RAW_COPPER", None, (5, 15)),
    ("RAW_TIN", None, (5, 16)),
    ("RAW_IRON", None, (5, 17)),
    ("COPPER", None, (5, 18)),
    ("TIN", None, (5, 19)),
    ("IRON", None, (5, 20)),
    ("VEGETAL_FIBER", None, (5, 21)),
    ("CLOTH", None, (5, 22)),
];

impl TileSheet {
    pub fn have_id(&self, id: &str) -> bool {
        self.appearances.get(id).is_some()
    }

    pub fn new(image: Image, tile_width: i16, tile_height: i16) -> Self {
        let mut sources: HashMap<SheetPosition, Rectangle<u16>> = HashMap::new();
        let max_row_i = image.height() as i16 / tile_height;
        let max_col_i = image.width() as i16 / tile_width;
        for tile_row_i in 0..max_row_i {
            for tile_col_i in 0..max_col_i {
                sources.insert(
                    (tile_row_i, tile_col_i),
                    Rectangle {
                        x: (tile_col_i * tile_height) as u16,
                        y: (tile_row_i * tile_width) as u16,
                        width: tile_width as u16,
                        height: tile_height as u16,
                    },
                );
            }
        }

        let mut appearances = HashMap::new();
        for (tile_id, back, fore) in APPEARANCES.iter() {
            appearances.insert(
                tile_id.to_string(),
                TileAppearance {
                    back: back.clone(),
                    fore: fore.clone(),
                },
            );
        }

        Self {
            image,
            sources,
            max_row_i: max_row_i - 1,
            max_col_i: max_col_i - 1,
            appearances,
            tile_width,
            tile_height,
        }
    }

    pub fn create_sprite_for(&self, tile_type_id: &str, x: i16, y: i16) -> Sprite {
        // FIXME BS NOW: retourner une liste de sprites (2 possible)
        let appearance = self
            .appearances
            .get(tile_type_id)
            .unwrap_or(self.appearances.get("UNKNOWN").unwrap());
        Sprite {
            source: self.sources[&appearance.fore],
            position: Point::new(x as f32, y as f32),
            scale: (1.0, 1.0),
        }
    }

    pub fn get_tile_width(&self) -> i16 {
        self.tile_width
    }

    pub fn get_tile_height(&self) -> i16 {
        self.tile_height
    }

    pub fn row_count(&self) -> i16 {
        self.max_row_i + 1
    }

    pub fn col_count(&self) -> i16 {
        self.max_col_i + 1
    }
}
