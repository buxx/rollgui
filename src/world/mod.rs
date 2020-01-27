use crate::error::RollingError;
use crate::util;
use crate::tile::world::Tiles;

pub mod level;
pub mod socket;


#[derive(Debug)]
pub struct WorldRow {
    pub cols: Vec<String>,
}


pub struct World {
    pub width: i32,
    pub height: i32,
    pub rows: Vec<WorldRow>,
}

impl World {
    pub fn new(world_raw: &str, tiles: &Tiles) -> Result<Self, RollingError> {
        let height = world_raw.lines().count() as i32;
        let longest_line = util::longest_line(world_raw);
        if !longest_line.is_some() {
            return Err(RollingError {
                message: String::from("There is no line in given world source"),
            });
        }

        let width = longest_line.unwrap().chars().count() as i32;
        let mut rows: Vec<WorldRow> = Vec::new();

        for line in world_raw.lines() {
            let mut cols: Vec<String> = Vec::new();

            for tile_char in line.chars() {
                let tile_id = tiles.tile_id(tile_char as u16);
                cols.push(tile_id);
            }

            let world_row = WorldRow { cols };
            rows.push(world_row);
        }

        Ok(
            Self {
                width,
                height,
                rows,
            }
        )
    }
}
