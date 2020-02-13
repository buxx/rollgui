use doryen_rs::{DoryenApi, UpdateEvent};
use doryen_ui as ui;

use crate::entity::player::Player;
use crate::gui::action;
use crate::gui::engine::Engine;
use crate::server::Server;
use crate::tile;
use crate::world;

pub struct WorldEngine {
    server: Server,
    player: Player,
    // Map coordinates where start to display it
    start_display_map_row_i: i32,
    start_display_map_col_i: i32,
    mouse_pos: (f32, f32),
}

impl WorldEngine {
    pub fn new(
        server: Server,
        player: Player,
        start_display_map_row_i: i32,
        start_display_map_col_i: i32,
    ) -> Self {
        Self {
            server,
            player,
            start_display_map_row_i,
            start_display_map_col_i,
            mouse_pos: (0.0, 0.0),
        }
    }
}

impl Engine for WorldEngine {
    fn get_name(&self) -> &str {
        "WORLD"
    }

    fn update(
        &mut self,
        api: &mut dyn DoryenApi,
        _width: i32,
        _height: i32,
    ) -> Option<action::Action> {
        self.mouse_pos = api.input().mouse_pos();
        None
    }

    fn render(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) {
        let con = api.con();

        for row_i in 0..height {
            for col_i in 0..width {
                let map_row_i = row_i + self.start_display_map_row_i;
                let map_col_i = col_i + self.start_display_map_col_i;

                // Pick map tile only is coordinate exist in map (can't pre-check height because
                // row can finish before end of complete width)
                if map_row_i < 0 || map_col_i < 0 || map_row_i >= self.server.world.height {
                    if let Some(default_id) = &self.server.world_tiles.default {
                        let default_appearance = self.server.world_tiles.appearance(default_id);
                        con.back(col_i as i32, row_i, default_appearance.back);
                        con.fore(col_i as i32, row_i, default_appearance.fore);
                        if default_appearance.ascii.is_some() {
                            con.ascii(
                                col_i as i32,
                                row_i,
                                default_appearance.ascii.unwrap() as u16,
                            );
                        }
                    }
                    continue;
                }
                let row = &self.server.world.rows[map_row_i as usize];

                // Can't pick tile if outside
                if map_col_i as usize >= row.cols.len() {
                    if let Some(default_id) = &self.server.world_tiles.default {
                        let default_appearance = self.server.world_tiles.appearance(default_id);
                        con.back(col_i as i32, row_i, default_appearance.back);
                        con.fore(col_i as i32, row_i, default_appearance.fore);
                        if default_appearance.ascii.is_some() {
                            con.ascii(
                                col_i as i32,
                                row_i,
                                default_appearance.ascii.unwrap() as u16,
                            );
                        }
                    }
                    continue;
                }

                let tile_id = &row.cols[map_col_i as usize];
                let appearance = self.server.world_tiles.appearance(&tile_id);

                con.back(col_i as i32, row_i, appearance.back);
                con.fore(col_i as i32, row_i, appearance.fore);
                if appearance.ascii.is_some() {
                    con.ascii(col_i as i32, row_i, appearance.ascii.unwrap() as u16);
                }
            }
        }

        con.back(
            self.mouse_pos.0 as i32,
            self.mouse_pos.1 as i32,
            (255, 255, 255, 255),
        );
    }

    fn resize(&mut self, _api: &mut dyn DoryenApi) {}

    fn teardown(&mut self) {}

    fn build_ui(
        &mut self,
        ctx: &mut ui::Context,
        width: i32,
        height: i32,
    ) -> Option<action::Action> {
        None
    }
}
