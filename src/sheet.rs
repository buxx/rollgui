use crate::tile::TileId;
use coffee::graphics::{Image, Point, Rectangle, Sprite};
use std::collections::HashMap;

pub type SheetPosition = (i16, i16);
pub type SheetPositions = [SheetPosition; 6];

#[derive(Debug, Clone)]
pub struct TileSheet {
    image: Image,
    pub sources: HashMap<SheetPosition, Rectangle<u16>>,
    max_row_i: i16,
    max_col_i: i16,
    appearances: HashMap<TileId, [SheetPosition; 6]>,
    tile_width: i16,
    tile_height: i16,
}

const APPEARANCES: [(&str, SheetPositions); 128] = [
    ("UNKNOWN", [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)]),
    ("SEA", [(7, 0), (7, 0), (7, 0), (7, 0), (7, 0), (7, 0)]),
    ("JUNGLE", [(7, 4), (7, 4), (7, 4), (7, 4), (7, 4), (7, 4)]),
    ("PLAIN", [(7, 2), (7, 2), (7, 2), (7, 2), (7, 2), (7, 2)]),
    ("HILL", [(7, 5), (7, 5), (7, 5), (7, 5), (7, 5), (7, 5)]),
    ("MOUNTAIN", [(7, 6), (7, 6), (7, 6), (7, 6), (7, 6), (7, 6)]),
    ("BEACH", [(7, 1), (7, 1), (7, 1), (7, 1), (7, 1), (7, 1)]),
    (
        "BACK_BEACH",
        [(0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3)],
    ),
    (
        "BACK_PLAIN",
        [(0, 6), (0, 6), (0, 6), (0, 6), (0, 6), (0, 6)],
    ),
    (
        "BACK_JUNGLE",
        [(0, 4), (0, 4), (0, 4), (0, 4), (0, 4), (0, 4)],
    ),
    (
        "BACK_HILL",
        [(0, 4), (0, 4), (0, 4), (0, 4), (0, 4), (0, 4)],
    ),
    (
        "BACK_MOUNTAIN",
        [(0, 5), (0, 5), (0, 5), (0, 5), (0, 5), (0, 5)],
    ),
    ("BACK_SEA", [(0, 2), (0, 2), (0, 2), (0, 2), (0, 2), (0, 2)]),
    ("SAND", [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)]),
    ("DRY_BUSH", [(2, 8), (2, 8), (2, 8), (2, 8), (2, 8), (2, 8)]),
    ("ROCK", [(2, 9), (2, 9), (2, 9), (2, 9), (2, 9), (2, 9)]),
    (
        "SEA_WATER",
        [(0, 12), (0, 12), (0, 12), (0, 13), (0, 13), (0, 13)],
    ),
    (
        "FRESH_WATER_TILE",
        [(0, 1), (0, 1), (0, 1), (0, 2), (0, 2), (0, 2)],
    ),
    (
        "SHORT_GRASS",
        [(1, 0), (1, 0), (1, 0), (1, 0), (1, 0), (1, 0)],
    ),
    (
        "HIGH_GRASS",
        [(1, 3), (1, 3), (1, 3), (1, 3), (1, 4), (1, 4)],
    ),
    (
        "ROCKY_GROUND",
        [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    ),
    ("DIRT", [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)]),
    (
        "LEAF_TREE",
        [(1, 7), (1, 7), (1, 7), (1, 7), (1, 7), (1, 7)],
    ),
    (
        "TROPICAL_TREE",
        [(1, 8), (1, 8), (1, 8), (1, 8), (1, 8), (1, 8)],
    ),
    (
        "DEAD_TREE",
        [(1, 9), (1, 9), (1, 9), (1, 9), (1, 9), (1, 9)],
    ),
    ("PLAYER", [(6, 0), (6, 0), (6, 0), (6, 0), (6, 0), (6, 0)]),
    (
        "PLAYER_LEFT",
        [(6, 1), (6, 1), (6, 1), (6, 1), (6, 1), (6, 1)],
    ),
    (
        "CHARACTER",
        [(6, 0), (6, 0), (6, 0), (6, 0), (6, 0), (6, 0)],
    ),
    (
        "STUFF_GENERIC",
        [(3, 0), (3, 0), (3, 0), (3, 0), (3, 0), (3, 0)],
    ),
    ("BOTTLE", [(3, 1), (3, 1), (3, 1), (3, 1), (3, 1), (3, 1)]),
    ("BAG", [(3, 2), (3, 2), (3, 2), (3, 2), (3, 2), (3, 2)]),
    ("COAT", [(3, 3), (3, 3), (3, 3), (3, 3), (3, 3), (3, 3)]),
    ("ARMOR", [(3, 4), (3, 4), (3, 4), (3, 4), (3, 4), (3, 4)]),
    ("SHIELD", [(3, 5), (3, 5), (3, 5), (3, 5), (3, 5), (3, 5)]),
    ("WEAPON", [(3, 6), (3, 6), (3, 6), (3, 6), (3, 6), (3, 6)]),
    ("SPEAR", [(3, 7), (3, 7), (3, 7), (3, 7), (3, 7), (3, 7)]),
    (
        "CORPSE",
        [(3, 11), (3, 11), (3, 11), (3, 11), (3, 11), (3, 11)],
    ),
    ("ANIMAL", [(3, 9), (3, 9), (3, 9), (3, 9), (3, 9), (3, 9)]),
    (
        "CRAFT",
        [(3, 10), (3, 10), (3, 10), (3, 10), (3, 10), (3, 10)],
    ),
    (
        "RESOURCE_GENERIC",
        [(5, 0), (5, 0), (5, 0), (5, 0), (5, 0), (5, 0)],
    ),
    (
        "COPPER_DEPOSIT",
        [(0, 7), (0, 7), (0, 7), (0, 7), (0, 7), (0, 7)],
    ),
    (
        "TIN_DEPOSIT",
        [(0, 8), (0, 8), (0, 8), (0, 8), (0, 8), (0, 8)],
    ),
    (
        "IRON_DEPOSIT",
        [(0, 9), (0, 9), (0, 9), (0, 9), (0, 9), (0, 9)],
    ),
    (
        "FRESH_WATER",
        [(5, 10), (5, 10), (5, 10), (5, 10), (5, 10), (5, 10)],
    ),
    (
        "SALTED_WATER",
        [(5, 10), (5, 10), (5, 10), (5, 10), (5, 10), (5, 10)],
    ),
    (
        "BEACH_SAND",
        [(5, 11), (5, 11), (5, 11), (5, 11), (5, 11), (5, 11)],
    ),
    ("SOIL", [(5, 9), (5, 9), (5, 9), (5, 9), (5, 9), (5, 9)]),
    ("WET_SOIL", [(5, 9), (5, 9), (5, 9), (5, 9), (5, 9), (5, 9)]),
    ("WOOD", [(5, 6), (5, 6), (5, 6), (5, 6), (5, 6), (5, 6)]),
    (
        "VEGETAL_FOOD_FRESH",
        [(5, 3), (5, 3), (5, 3), (5, 3), (5, 3), (5, 3)],
    ),
    (
        "SHELLFISH_FRESH",
        [(5, 1), (5, 1), (5, 1), (5, 1), (5, 1), (5, 1)],
    ),
    (
        "FISH_FRESH",
        [(5, 1), (5, 1), (5, 1), (5, 1), (5, 1), (5, 1)],
    ),
    ("RAW_MEAT", [(5, 4), (5, 4), (5, 4), (5, 4), (5, 4), (5, 4)]),
    (
        "COOKED_MEAT",
        [(5, 5), (5, 5), (5, 5), (5, 5), (5, 5), (5, 5)],
    ),
    (
        "SMOKED_MEAT",
        [(5, 5), (5, 5), (5, 5), (5, 5), (5, 5), (5, 5)],
    ),
    (
        "ANIMAL_SKIN",
        [(5, 8), (5, 8), (5, 8), (5, 8), (5, 8), (5, 8)],
    ),
    (
        "GRAMINEAE",
        [(5, 13), (5, 13), (5, 13), (5, 13), (5, 13), (5, 13)],
    ),
    (
        "BREAD",
        [(5, 14), (5, 14), (5, 14), (5, 14), (5, 14), (5, 14)],
    ),
    (
        "RAW_STONE",
        [(5, 12), (5, 12), (5, 12), (5, 12), (5, 12), (5, 12)],
    ),
    (
        "LEATHER_PIECE",
        [(5, 7), (5, 7), (5, 7), (5, 7), (5, 7), (5, 7)],
    ),
    (
        "BUILD_GENERIC",
        [(4, 1), (4, 1), (4, 1), (4, 1), (4, 1), (4, 1)],
    ),
    (
        "CAMPFIRE__OFF",
        [(9, 0), (9, 0), (9, 0), (9, 0), (9, 0), (9, 0)],
    ),
    ("CAMPFIRE", [(9, 1), (9, 2), (9, 3), (9, 4), (9, 1), (9, 2)]),
    ("WALL", [(4, 2), (4, 2), (4, 2), (4, 2), (4, 2), (4, 2)]),
    (
        "WOOD_FENCE",
        [(4, 2), (4, 2), (4, 2), (4, 2), (4, 2), (4, 2)],
    ),
    (
        "STONE_WALL",
        [(4, 3), (4, 3), (4, 3), (4, 3), (4, 3), (4, 3)],
    ),
    ("LOOM", [(4, 6), (4, 6), (4, 6), (4, 6), (4, 6), (4, 6)]),
    (
        "BRUSHWOOD_EDGE",
        [(4, 4), (4, 4), (4, 4), (4, 4), (4, 4), (4, 4)],
    ),
    (
        "SOIL_WALL",
        [(4, 5), (4, 5), (4, 5), (4, 5), (4, 5), (4, 5)],
    ),
    (
        "BASKETRY_BAG",
        [(3, 12), (3, 12), (3, 12), (3, 12), (3, 12), (3, 12)],
    ),
    (
        "SKIN_BAG",
        [(3, 14), (3, 14), (3, 14), (3, 14), (3, 14), (3, 14)],
    ),
    (
        "LEATHER_BAG",
        [(3, 13), (3, 13), (3, 13), (3, 13), (3, 13), (3, 13)],
    ),
    (
        "TRAVOIS",
        [(3, 20), (3, 20), (3, 20), (3, 20), (3, 20), (3, 20)],
    ),
    (
        "CLOTH_BAG",
        [(3, 15), (3, 15), (3, 15), (3, 15), (3, 15), (3, 15)],
    ),
    (
        "ANIMAL_SKIN_CLOTHES",
        [(3, 17), (3, 17), (3, 17), (3, 17), (3, 17), (3, 17)],
    ),
    (
        "LEATHER_CLOTHES",
        [(3, 16), (3, 16), (3, 16), (3, 16), (3, 16), (3, 16)],
    ),
    (
        "LEATHER_BRIGANDINE",
        [(3, 19), (3, 19), (3, 19), (3, 19), (3, 19), (3, 19)],
    ),
    (
        "HARE",
        [(3, 26), (3, 26), (3, 26), (3, 26), (3, 26), (3, 26)],
    ),
    (
        "PIG",
        [(3, 25), (3, 25), (3, 25), (3, 25), (3, 25), (3, 25)],
    ),
    (
        "GOAT",
        [(3, 24), (3, 24), (3, 24), (3, 24), (3, 24), (3, 24)],
    ),
    (
        "MOORHEN",
        [(3, 23), (3, 23), (3, 23), (3, 23), (3, 23), (3, 23)],
    ),
    (
        "CRAB",
        [(3, 22), (3, 22), (3, 22), (3, 22), (3, 22), (3, 22)],
    ),
    (
        "RAW_BRICK",
        [(3, 27), (3, 27), (3, 27), (3, 27), (3, 27), (3, 27)],
    ),
    (
        "FIRED_BRICK",
        [(3, 28), (3, 28), (3, 28), (3, 28), (3, 28), (3, 28)],
    ),
    (
        "RAW_BRICK_WALL",
        [(4, 7), (4, 7), (4, 7), (4, 7), (4, 7), (4, 7)],
    ),
    (
        "FIRED_BRICK_WALL",
        [(4, 8), (4, 8), (4, 8), (4, 8), (4, 8), (4, 8)],
    ),
    (
        "SOIL_KILN__OFF",
        [(10, 0), (10, 0), (10, 0), (10, 0), (10, 0), (10, 0)],
    ),
    (
        "SOIL_KILN",
        [(10, 1), (10, 2), (10, 1), (10, 2), (10, 1), (10, 2)],
    ),
    (
        "RAW_BRICK_KILN__OFF",
        [(11, 0), (11, 0), (11, 0), (11, 0), (11, 0), (11, 0)],
    ),
    (
        "RAW_BRICK_KILN",
        [(11, 1), (11, 2), (11, 1), (11, 2), (11, 1), (11, 2)],
    ),
    (
        "FIRED_BRICK_KILN__OFF",
        [(12, 0), (12, 0), (12, 0), (12, 0), (12, 0), (12, 0)],
    ),
    (
        "FIRED_BRICK_KILN",
        [(12, 1), (12, 2), (12, 1), (12, 2), (12, 1), (12, 2)],
    ),
    (
        "RAW_COPPER",
        [(5, 15), (5, 15), (5, 15), (5, 15), (5, 15), (5, 15)],
    ),
    (
        "RAW_TIN",
        [(5, 16), (5, 16), (5, 16), (5, 16), (5, 16), (5, 16)],
    ),
    (
        "RAW_IRON",
        [(5, 17), (5, 17), (5, 17), (5, 17), (5, 17), (5, 17)],
    ),
    (
        "COPPER",
        [(5, 18), (5, 18), (5, 18), (5, 18), (5, 18), (5, 18)],
    ),
    (
        "TIN",
        [(5, 19), (5, 19), (5, 19), (5, 19), (5, 19), (5, 19)],
    ),
    (
        "IRON",
        [(5, 20), (5, 20), (5, 20), (5, 20), (5, 20), (5, 20)],
    ),
    (
        "VEGETAL_FIBER",
        [(5, 21), (5, 21), (5, 21), (5, 21), (5, 21), (5, 21)],
    ),
    (
        "CLOTH",
        [(5, 22), (5, 22), (5, 22), (5, 22), (5, 22), (5, 22)],
    ),
    (
        "GROUND",
        [(0, 10), (0, 10), (0, 10), (0, 10), (0, 10), (0, 10)],
    ),
    (
        "RAW_CLAY_FLOOR",
        [(0, 10), (0, 10), (0, 10), (0, 10), (0, 10), (0, 10)],
    ),
    (
        "DOOR",
        [(4, 12), (4, 12), (4, 12), (4, 12), (4, 12), (4, 12)],
    ),
    (
        "PLOUGHED_LAND",
        [(0, 11), (0, 11), (0, 11), (0, 11), (0, 11), (0, 11)],
    ),
    (
        "SEEDS",
        [(1, 10), (1, 10), (1, 10), (1, 10), (1, 10), (1, 10)],
    ),
    (
        "CEREAL",
        [(5, 13), (5, 13), (5, 13), (5, 13), (5, 13), (5, 13)],
    ),
    (
        "GROW_PROGRESS_0",
        [(8, 0), (8, 0), (8, 0), (8, 0), (8, 0), (8, 0)],
    ),
    (
        "GROW_PROGRESS_1",
        [(8, 1), (8, 1), (8, 1), (8, 1), (8, 1), (8, 1)],
    ),
    (
        "GROW_PROGRESS_2",
        [(8, 2), (8, 2), (8, 2), (8, 2), (8, 2), (8, 2)],
    ),
    (
        "GROW_PROGRESS_3",
        [(8, 3), (8, 3), (8, 3), (8, 3), (8, 3), (8, 3)],
    ),
    (
        "GROW_PROGRESS_4",
        [(8, 4), (8, 4), (8, 4), (8, 4), (8, 4), (8, 4)],
    ),
    (
        "GROW_PROGRESS_CEREAL_0",
        [(8, 0), (8, 0), (8, 0), (8, 0), (8, 0), (8, 0)],
    ),
    (
        "GROW_PROGRESS_CEREAL_1",
        [(8, 1), (8, 1), (8, 1), (8, 1), (8, 1), (8, 1)],
    ),
    (
        "GROW_PROGRESS_CEREAL_2",
        [(8, 2), (8, 2), (8, 2), (8, 2), (8, 2), (8, 2)],
    ),
    (
        "GROW_PROGRESS_CEREAL_3",
        [(8, 3), (8, 3), (8, 3), (8, 3), (8, 3), (8, 3)],
    ),
    (
        "GROW_PROGRESS_CEREAL_4",
        [(8, 4), (8, 4), (8, 4), (8, 4), (8, 4), (8, 4)],
    ),
    (
        "FLOOR",
        [(5, 23), (5, 23), (5, 23), (5, 23), (5, 23), (5, 23)],
    ),
    (
        "BREAD",
        [(5, 24), (5, 24), (5, 24), (5, 24), (5, 24), (5, 24)],
    ),
    (
        "WOOL",
        [(5, 25), (5, 25), (5, 25), (5, 25), (5, 25), (5, 25)],
    ),
    (
        "MORTIER_PILON",
        [(5, 26), (5, 26), (5, 26), (5, 26), (5, 26), (5, 26)],
    ),
    (
        "CHARCOAL",
        [(5, 27), (5, 27), (5, 27), (5, 27), (5, 27), (5, 27)],
    ),
    (
        "ROUET",
        [(5, 28), (5, 28), (5, 28), (5, 28), (5, 28), (5, 28)],
    ),
    (
        "SPINDLE",
        [(3, 29), (5, 29), (5, 29), (5, 29), (5, 29), (5, 29)],
    ),
    (
        "LITTLE_FISHING_NET",
        [(5, 29), (5, 29), (5, 29), (5, 29), (5, 29), (5, 29)],
    ),
    (
        "STONE_ANVIL",
        [(5, 30), (5, 30), (5, 30), (5, 30), (5, 30), (5, 30)],
    ),
    (
        "IRON_ANVIL",
        [(5, 31), (5, 31), (5, 31), (5, 31), (5, 31), (5, 31)],
    ),
    (
        "COLLECT",
        [(2, 10), (2, 10), (2, 10), (2, 10), (2, 10), (2, 10)],
    ),
    (
        "HARVEST",
        [(2, 11), (2, 11), (2, 11), (2, 11), (2, 11), (2, 11)],
    ),
];

impl TileSheet {
    pub fn have_id(&self, id: &str) -> bool {
        self.appearances.get(id).is_some()
    }

    pub fn appearance(&self, id: &str) -> Option<SheetPositions> {
        if let Some(positions) = self.appearances.get(id) {
            Some(*positions)
        } else {
            None
        }
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
        for (tile_id, sprites) in APPEARANCES.iter() {
            appearances.insert(tile_id.to_string(), sprites.clone());
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

    pub fn create_sprite_for(&self, tile_type_id: &str, x: i16, y: i16, sprite_i: i32) -> Sprite {
        let appearance = self
            .appearances
            .get(tile_type_id)
            .unwrap_or(self.appearances.get("UNKNOWN").unwrap());
        let sheet_position = appearance[sprite_i as usize];
        Sprite {
            source: self.sources[&sheet_position],
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
