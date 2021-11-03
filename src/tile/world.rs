use crate::error::RollingError;
use crate::sheet::SheetPosition;
use crate::tile::TileId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Tiles {
    appearances: HashMap<TileId, [SheetPosition; 6]>,
    codes: HashMap<u16, TileId>,
    pub default: Option<TileId>,
}

// FIXME: not used
const APPEARANCES: [(&str, [SheetPosition; 6]); 6] = [
    ("JUNGLE", [(0, 7), (0, 7), (0, 7), (0, 7), (0, 7), (0, 7)]),
    ("PLAIN", [(0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1)]),
    (
        "HILL",
        [(0, 11), (0, 11), (0, 11), (0, 11), (0, 11), (0, 11)],
    ),
    (
        "MOUNTAIN",
        [(0, 22), (0, 22), (0, 22), (0, 22), (0, 22), (0, 22)],
    ),
    ("SEA", [(0, 8), (0, 8), (0, 8), (0, 8), (0, 8), (0, 8)]),
    ("BEACH", [(0, 2), (0, 2), (0, 2), (0, 2), (0, 2), (0, 2)]),
];

impl Tiles {
    pub fn new(legend: &str) -> Result<Self, RollingError> {
        let default_appearances: HashMap<&str, [SheetPosition; 6]> =
            APPEARANCES.iter().cloned().collect();
        let mut default_tile_id: Option<TileId> = None;

        let mut codes: HashMap<u16, TileId> = HashMap::new();
        let mut appearances: HashMap<TileId, [SheetPosition; 6]> = HashMap::new();

        for line in legend.lines() {
            let mut split = line.split_ascii_whitespace();
            let char_ = split.next().unwrap().trim().chars().nth(0).unwrap() as u16;
            let mut id = split.next().unwrap().trim();
            if id.ends_with("*") {
                id = id.trim_end_matches("*");
                default_tile_id = Some(id.to_string());
            }
            let mut appearance = [(0, 35), (0, 35), (0, 35), (0, 35), (0, 35), (0, 35)];
            if let Some(new_appearance) = default_appearances.get(id) {
                appearance = new_appearance.clone()
            }

            codes.insert(char_, id.to_string());
            appearances.insert(id.to_string(), appearance);
        }

        Ok(Tiles {
            appearances,
            codes,
            default: default_tile_id,
        })
    }

    pub fn appearance(&self, tile_id: &str) -> &[SheetPosition; 6] {
        self.appearances.get(tile_id).unwrap()
    }

    pub fn tile_id(&self, code: u16) -> String {
        self.codes.get(&code).unwrap().clone()
    }
}
