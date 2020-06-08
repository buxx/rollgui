use crate::error::RollingError;
use crate::tile::zone::Tiles;
use crate::util;

#[derive(Debug)]
pub struct LevelRow {
    pub cols: Vec<String>,
}

#[derive(Debug)]
pub struct Level {
    pub width: i32,
    pub height: i32,
    pub rows: Vec<LevelRow>,
    pub world_tile_type_id: String,
}

impl Level {
    pub fn new(
        zone_raw: &str,
        tiles: &Tiles,
        world_tile_type_id: String,
    ) -> Result<Self, RollingError> {
        let height = zone_raw.lines().count() as i32;
        let longest_line = util::longest_line(zone_raw);
        if !longest_line.is_some() {
            return Err(RollingError {
                message: String::from("There is no line in given zone source"),
            });
        }

        let width = longest_line.unwrap().chars().count() as i32;
        let mut rows: Vec<LevelRow> = Vec::new();

        for line in zone_raw.lines() {
            let mut cols: Vec<String> = Vec::new();

            for tile_char in line.chars() {
                let tile_id = tiles.tile_id(tile_char as u16);
                cols.push(tile_id);
            }

            let level_row = LevelRow { cols };
            rows.push(level_row);
        }

        Ok(Self {
            width,
            height,
            rows,
            world_tile_type_id,
        })
    }

    // pub fn render(
    //     &mut self,
    //     api: &mut dyn DoryenApi,
    //     tiles: &Tiles,
    //     start_display_map_row_i: i32,
    //     start_display_map_col_i: i32,
    //     display_width: i32,
    //     display_height: i32,
    //     row_offset: i32,
    //     col_offset: i32,
    // ) {
    //     let con = api.con();
    //
    //     for row_i in 0..display_height {
    //         for col_i in 0..display_width {
    //             let map_row_i = row_i + start_display_map_row_i;
    //             let map_col_i = col_i + start_display_map_col_i;
    //
    //             // Pick map tile only is coordinate exist in map (can't pre-check height because
    //             // row can finish before end of complete width)
    //             if map_row_i < 0 || map_col_i < 0 || map_row_i >= self.height {
    //                 continue;
    //             }
    //             let row = &self.rows[map_row_i as usize];
    //
    //             // Can't pick tile if outside
    //             if map_col_i as usize >= row.cols.len() {
    //                 continue;
    //             }
    //
    //             let tile_id = &row.cols[map_col_i as usize];
    //             let appearance = tiles.appearance(&tile_id);
    //
    //             con.back(
    //                 col_i as i32 + col_offset,
    //                 row_i + row_offset,
    //                 appearance.back,
    //             );
    //             con.fore(
    //                 col_i as i32 + col_offset,
    //                 row_i + row_offset,
    //                 appearance.fore,
    //             );
    //             if appearance.ascii.is_some() {
    //                 con.ascii(
    //                     col_i as i32 + col_offset,
    //                     row_i + row_offset,
    //                     appearance.ascii.unwrap() as u16,
    //                 );
    //             }
    //         }
    //     }
    // }

    // row_i, col_i
    pub fn tile_id(&self, row_i: i16, col_i: i16) -> String {
        if col_i < 0 || row_i < 0 {
            return String::from("NOTHING");
        }

        if row_i >= self.rows.len() as i16 {
            return String::from("NOTHING");
        }

        let row = &self.rows[row_i as usize];

        if col_i >= row.cols.len() as i16 {
            return String::from("NOTHING");
        }

        row.cols[col_i as usize].clone()
    }

    pub fn get_successors(&self, tiles: &Tiles, row_i: i16, col_i: i16) -> Vec<((i16, i16), u32)> {
        let mut successors = vec![];

        for (modifier_row_i, modifier_col_i) in [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            // no center pos
            (0, 1),
            (1, 1),
            (1, -1),
            (1, 0),
        ]
        .iter()
        {
            let new_row_i = row_i + *modifier_row_i;
            let new_col_i = col_i + *modifier_col_i;
            if tiles.browseable(&self.tile_id(new_row_i, new_col_i)) {
                successors.push(((new_row_i, new_col_i), 1));
            }
        }

        successors
    }
}
