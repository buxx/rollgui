use doryen_rs::{Color};
use std::collections::HashMap;

use crate::color;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Tile {
    Empty,
    SaltedWater,
    Sand,
    DryBush,
}

#[derive(Debug)]
pub struct TileAppearance {
    pub back: Color,
    pub fore: Color,
    pub ascii: Option<char>,
}

#[derive(Debug)]
pub struct Tiles {
    appearances: HashMap<Tile, TileAppearance>,
    codes: HashMap<char, Tile>,
    browseables: HashMap<Tile, bool>,
}

impl Tiles {
    pub fn new() -> Self {
        let mut appearances = HashMap::new();
        let mut codes = HashMap::new();
        let mut browseables = HashMap::new();

        codes.insert('d', Tile::SaltedWater);
        appearances.insert(Tile::SaltedWater, TileAppearance {
            back: color::BLACK,
            fore: color::BLUE,
            ascii: Some('~'),
        });

        codes.insert('s', Tile::Sand);
        appearances.insert(Tile::Sand, TileAppearance {
            back: color::OLIVE,
            fore: color::WHITE,
            ascii: None,
        });
        browseables.insert(Tile::Sand, true);

        codes.insert('ʛ', Tile::DryBush);
        appearances.insert(Tile::DryBush, TileAppearance {
            back: color::OLIVE,
            fore: color::BLACK,
            ascii: Some('#'),
        });
        browseables.insert(Tile::DryBush, true);

        Tiles {
            appearances,
            codes,
            browseables,
        }
    }

    pub fn appearance(&self, tile: &Tile) -> &TileAppearance {
        self.appearances.get(tile).unwrap()
    }

    pub fn tile(&self, code: char) -> Tile {
        self.codes.get(&code).unwrap().clone()
    }

    pub fn browseable(&self, tile: &Tile) -> bool {
        if let Some(browseable) = self.browseables.get(tile) {
            return *browseable
        }
        false
    }
}
