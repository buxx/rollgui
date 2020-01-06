use doryen_rs::{DoryenApi, Engine, UpdateEvent, TextAlign};
use doryen_ui as ui;

use std::path::Path;
use crate::color;
use crate::entity::player::{Player};
use crate::zone::level::{Level};
use crate::util;
use crate::tile::{Tiles};
use std::error::Error;


pub struct RollingGui {
    player: Player,
    level: Level,
    ctx: ui::Context,
    width: u32,
    height: u32,
    tiles: Tiles,
    mouse_pos: (f32, f32),
    // Map coordinates where start to display it
    start_display_map_row_i: i32,
    start_display_map_col_i: i32,
}

impl RollingGui {
    pub fn new(width: i32, height: i32) -> Result<Self, Box<dyn Error>> {
        let tiles= Tiles::new();
        let zone_file_content = util::get_file_content(
            Path::new("static/zone_test.txt")
        )?;
        let zone_raw = util::extract_block_from_source(
            util::BLOCK_GEO,
            &zone_file_content,
        )?;
        let level = Level::new(&zone_raw, &tiles)?;

        // TODO: from server
        let player_row_i = 0;
        let player_col_i = 1;

        // Compute display positions (player at center of display)
        let start_display_map_row_i: i32 = player_row_i - (height / 2);
        let start_display_map_col_i: i32 = player_col_i - (width / 2);

        Ok(
            Self {
                player: Player::new((player_row_i, player_col_i)),
                level,
                ctx: ui::Context::new(),
                width: width as u32,
                height: height as u32,
                tiles,
                start_display_map_row_i,
                start_display_map_col_i,
                mouse_pos: (0.0, 0.0),
            }
        )
    }

    pub fn clear_con(&self, api: &mut dyn DoryenApi) {
        let con = api.con();
        con.clear(Some(color::BLACK), Some(color::BLACK), Some(' ' as u16));
    }

    pub fn can_move(&self, position: (i32, i32)) -> bool {
        let tile = self.level.tile(position);
        self.tiles.browseable(&tile)
    }
    fn build_ui(&mut self) {

    }
}

impl Engine for RollingGui {
    fn init(&mut self, api: &mut dyn DoryenApi) {
        api.con().register_color("white", color::WHITE);
    }
    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        let input = api.input();

        let mut mov = self.player.move_from_input(api);
        let mut coef = 1.0 / std::f32::consts::SQRT_2;

        if !self.can_move(self.player.next_position((mov.0, 0))) {
            mov.0 = 0;
            coef = 1.0;
        }
        if !self.can_move(self.player.next_position((0, mov.1))) {
            mov.1 = 0;
            coef = 1.0;
        }
        self.player.move_by(mov, coef);
        dbg!(self.player.position);

        self.start_display_map_row_i = self.player.position.0 as i32 - (self.height as i32 / 2);
        self.start_display_map_col_i = self.player.position.1 as i32 - (self.width as i32 / 2);

        self.mouse_pos = api.input().mouse_pos();

        ui::update_doryen_input_data(api, &mut self.ctx);
        self.build_ui();

        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi) {
        self.clear_con(api);
        self.level.render(api, &self.tiles, self.start_display_map_row_i, self.start_display_map_col_i, self.width, self.height);
        self.player.render(api, self.width as i32, self.height as i32);

        let fps = api.fps();
        api.con().print_color(
            1,
            20,
            &format!("row {} / col {} {}fps", self.mouse_pos.1 as i32, self.mouse_pos.0 as i32, fps),
            TextAlign::Left,
            None,
        );
        api.con().back(self.mouse_pos.0 as i32, self.mouse_pos.1 as i32, (255, 255, 255, 255));
        ui::render_doryen(api.con(), &mut self.ctx);
    }
    fn resize(&mut self, api: &mut dyn DoryenApi) {
        self.width = api.get_screen_size().0 / 8;
        self.height = api.get_screen_size().1 / 8;
        api.con().resize(self.width, self.height);
    }
}