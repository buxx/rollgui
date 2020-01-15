use doryen_rs::{Color as dColor};
use std::collections::HashMap;
use colors_transform::{Rgb, Color};
use serde_json::Value;

use crate::color;


#[derive(Debug)]
pub struct TileAppearance {
    pub back: dColor,
    pub fore: dColor,
    pub ascii: Option<char>,
}

#[derive(Debug)]
pub struct Tiles {
    appearances: HashMap<String, TileAppearance>,
    codes: HashMap<char, String>,
    browseables: HashMap<String, bool>,
}

const ASCII_MAP: [(char, char); 9] = [
    ('⡩', '.'),
    ('ʛ', 'G'),
    ('܄', '`'),
    ('፨', '.'),
    ('⁖', '.'),
    ('߉', 'p'),
    ('ፆ', 'P'),
    ('آ', 'I'),
    ('ގ', '~'),
];

// TODO: compute it automatically
const COLOR_MAP: [(&str, &str); 3] = [
    ("", "#000"),
    ("g31", "#4D4D4D"),
    ("g35", "#989898"),
];

pub const NOTHING: &str = "NOTHING";
pub const UNKNOWN: &str = "UNKNOWN";

impl Tiles {
    pub fn new(data: Value) -> Self {
        let mut appearances = HashMap::new();
        let mut codes = HashMap::new();
        let mut browseables = HashMap::new();

        let mut ascii_map: HashMap<char, char> = HashMap::new();
        for (char_key, new_char) in ASCII_MAP.into_iter() {
            ascii_map.insert(*char_key, *new_char);
        }

        let mut color_map: HashMap<&str, &str> = HashMap::new();
        for (from_color, to_color) in COLOR_MAP.into_iter() {
            color_map.insert(*from_color, *to_color);
        }

        for tile_value in data.as_array().unwrap() {
            let mut back_hex = tile_value["background_high_color"].as_str().unwrap();
            if let Some(new_color) = color_map.get(back_hex) { back_hex = new_color; }
            let back_rgb =  Rgb::from_hex_str(back_hex).unwrap();
            let back_color: dColor = (back_rgb.get_red() as u8, back_rgb.get_green() as u8, back_rgb.get_blue() as u8, 255);

            let mut fore_hex = tile_value["foreground_high_color"].as_str().unwrap();
            if let Some(new_color) = color_map.get(fore_hex) { fore_hex = new_color; }
            let fore_rgb =  Rgb::from_hex_str(fore_hex).unwrap();
            let fore_color: dColor = (fore_rgb.get_red() as u8, fore_rgb.get_green() as u8, fore_rgb.get_blue() as u8, 255);

            let id = tile_value["id"].as_str().unwrap();
            let mut ch: char = tile_value["char"].as_str().unwrap().chars().nth(0).unwrap();
            let code = ch;
            if let Some(new_ch) = ascii_map.get(&ch) {
                ch = *new_ch;
            }

            codes.insert(code, String::from(id));
            appearances.insert(String::from(id), TileAppearance {
                back: back_color,
                fore: fore_color,
                ascii: Some(ch),
            });
            // TODO evolve browseables schema (WALKING, etc)
            if let Some(traversable) = tile_value["traversable"].as_object() {
                if let Some(walking) = traversable.get("WALKING") {
                    if let Some(can_walk) = walking.as_bool() {
                        browseables.insert(String::from(id), can_walk);
                    }
                }
            }
        }

        // Default display for UNKNOWN
        codes.insert('?', String::from(UNKNOWN));
        appearances.insert(String::from(UNKNOWN), TileAppearance {
            back: color::BLACK,
            fore: (102, 102, 153, 255),
            ascii: Some('?'),
        });

        Tiles {
            appearances,
            codes,
            browseables,
        }
    }

    pub fn appearance(&self, tile_id: &str) -> &TileAppearance {
        self.appearances.get(tile_id).unwrap()
    }

    pub fn tile_id(&self, code: char) -> String {
        if let Some(tile_id) = self.codes.get(&code) {
            return String::from(tile_id)
        }
        String::from(UNKNOWN)
    }

    pub fn browseable(&self, tile_id: &str) -> bool {
        if let Some(browseable) = self.browseables.get(tile_id) {
            return *browseable
        }
        false
    }
}
