use doryen_rs::Color as dColor;
use std::collections::HashMap;

use crate::color;
use crate::error::RollingError;
use crate::gui;

#[derive(Debug, Clone, Copy)]
pub struct TileAppearance {
    pub back: dColor,
    pub fore: dColor,
    pub ascii: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct Tiles {
    appearances: HashMap<String, TileAppearance>,
    codes: HashMap<u16, String>,
    pub default: Option<String>,
}

const APPEARANCES: [(&str, TileAppearance); 6] = [
    (
        "JUNGLE",
        TileAppearance {
            back: color::BLACK,
            fore: color::GREEN,
            ascii: Some(gui::CHAR_TROPICAL_TREE),
        },
    ),
    (
        "PLAIN",
        TileAppearance {
            back: color::BLACK,
            fore: color::FADE_YELLOW,
            ascii: Some(gui::CHAR_GRASS),
        },
    ),
    (
        "HILL",
        TileAppearance {
            back: color::BLACK,
            fore: color::FADE_GREEN,
            ascii: Some(gui::CHAR_GEARS),
        },
    ),
    (
        "MOUNTAIN",
        TileAppearance {
            back: color::BLACK,
            fore: color::LIGHT_GRAY,
            ascii: Some(gui::CHAR_GEARS),
        },
    ),
    (
        "SEA",
        TileAppearance {
            back: color::BLACK,
            fore: color::BLUE,
            ascii: Some(gui::CHAR_DEEP_WATER),
        },
    ),
    (
        "BEACH",
        TileAppearance {
            back: color::BLACK,
            fore: color::YELLOW,
            ascii: Some(gui::CHAR_GRASS),
        },
    ),
];

impl Tiles {
    pub fn new(legend: &str) -> Result<Self, RollingError> {
        let default_appearances: HashMap<&str, TileAppearance> =
            APPEARANCES.iter().cloned().collect();
        let mut default_tile_id: Option<String> = None;

        let mut codes: HashMap<u16, String> = HashMap::new();
        let mut appearances: HashMap<String, TileAppearance> = HashMap::new();

        for line in legend.lines() {
            let mut split = line.split_ascii_whitespace();
            let char_ = split.next().unwrap().trim().chars().nth(0).unwrap() as u16;
            let mut id = split.next().unwrap().trim();
            if id.ends_with("*") {
                id = id.trim_end_matches("*");
                default_tile_id = Some(id.to_string());
            }
            let mut appearance = TileAppearance {
                back: color::BLACK,
                fore: color::WHITE,
                ascii: Some('?' as u16),
            };
            if let Some(new_appearance) = default_appearances.get(id) {
                appearance = *new_appearance
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
