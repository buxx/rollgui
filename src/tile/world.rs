use crate::error::RollingError;
use crate::tile::{TileAppearance, TileId};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Tiles {
    appearances: HashMap<TileId, TileAppearance>,
    codes: HashMap<u16, TileId>,
    pub default: Option<TileId>,
}

const APPEARANCES: [(&str, TileAppearance); 6] = [
    (
        "JUNGLE",
        TileAppearance {
            back: None,
            fore: (0, 7),
        },
    ),
    (
        "PLAIN",
        TileAppearance {
            back: None,
            fore: (0, 1),
        },
    ),
    (
        "HILL",
        TileAppearance {
            back: None,
            fore: (0, 11),
        },
    ),
    (
        "MOUNTAIN",
        TileAppearance {
            back: None,
            fore: (0, 22),
        },
    ),
    (
        "SEA",
        TileAppearance {
            back: None,
            fore: (0, 8),
        },
    ),
    (
        "BEACH",
        TileAppearance {
            back: None,
            fore: (0, 2),
        },
    ),
];

impl Tiles {
    pub fn new(legend: &str) -> Result<Self, RollingError> {
        let default_appearances: HashMap<&str, TileAppearance> =
            APPEARANCES.iter().cloned().collect();
        let mut default_tile_id: Option<TileId> = None;

        let mut codes: HashMap<u16, TileId> = HashMap::new();
        let mut appearances: HashMap<TileId, TileAppearance> = HashMap::new();

        for line in legend.lines() {
            let mut split = line.split_ascii_whitespace();
            let char_ = split.next().unwrap().trim().chars().nth(0).unwrap() as u16;
            let mut id = split.next().unwrap().trim();
            if id.ends_with("*") {
                id = id.trim_end_matches("*");
                default_tile_id = Some(id.to_string());
            }
            let mut appearance = TileAppearance {
                back: None,
                fore: (0, 35),
            };
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

    pub fn appearance(&self, tile_id: &str) -> &TileAppearance {
        self.appearances.get(tile_id).unwrap()
    }

    pub fn tile_id(&self, code: u16) -> String {
        self.codes.get(&code).unwrap().clone()
    }
}
