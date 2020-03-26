use doryen_rs::DoryenApi;
use serde::{Deserialize, Serialize};

use crate::color;
use crate::gui;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: String,
    // row_i, col_i
    pub position: (f32, f32),
    pub world_position: (i32, i32),
    speed: f32,
    pub max_life_comp: f32,
    pub life_points: f32,
    pub action_points: f32,
    pub feel_thirsty: bool,
    pub feel_hungry: bool,
    pub weight_overcharge: bool,
    pub clutter_overcharge: bool,
    pub unread_event: bool,
    pub unread_zone_message: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiCharacter {
    pub id: String,
    pub zone_row_i: i32,
    pub zone_col_i: i32,
    pub world_row_i: i32,
    pub world_col_i: i32,
    pub max_life_comp: f32,
    pub life_points: f32,
    pub action_points: f32,
    pub feel_thirsty: bool,
    pub feel_hungry: bool,
    pub weight_overcharge: bool,
    pub clutter_overcharge: bool,
    pub unread_event: bool,
    pub unread_zone_message: bool,
}

impl Player {
    pub fn new(
        id: &str,
        position: (i32, i32),
        world_position: (i32, i32),
        max_life_comp: f32,
        life_points: f32,
        action_points: f32,
        feel_thirsty: bool,
        feel_hungry: bool,
        weight_overcharge: bool,
        clutter_overcharge: bool,
        unread_event: bool,
        unread_zone_message: bool,
    ) -> Self {
        Self {
            id: id.to_string(),
            position: (position.0 as f32, position.1 as f32),
            world_position: (world_position.0, world_position.1),
            speed: 0.2,
            max_life_comp,
            life_points,
            action_points,
            feel_thirsty,
            feel_hungry,
            weight_overcharge,
            clutter_overcharge,
            unread_event,
            unread_zone_message,
        }
    }

    // row_i, col_i
    pub fn move_from_input(&self, api: &mut dyn DoryenApi) -> (i32, i32) {
        let input = api.input();
        let mut mov = (0, 0);
        if input.key("ArrowLeft") || input.key("KeyA") {
            mov.1 = -1;
        } else if input.key("ArrowRight") || input.key("KeyD") {
            mov.1 = 1;
        }
        if input.key("ArrowUp") || input.key("KeyW") {
            mov.0 = -1;
        } else if input.key("ArrowDown") || input.key("KeyS") {
            mov.0 = 1;
        }
        mov
    }

    pub fn move_to(&mut self, pos: (i32, i32)) {
        self.position = (pos.0 as f32, pos.1 as f32);
    }

    pub fn move_by(&mut self, mov: (i32, i32), coef: f32) -> bool {
        let old_row_i = self.position.0 as i32;
        let old_col_i = self.position.1 as i32;
        self.position.0 += self.speed * mov.0 as f32 * coef;
        self.position.1 += self.speed * mov.1 as f32 * coef;
        old_row_i != self.position.0 as i32 || old_col_i != self.position.1 as i32
    }

    pub fn next_position(&self, mov: (i32, i32)) -> (i32, i32) {
        (
            self.position.0 as i32 + mov.0,
            self.position.1 as i32 + mov.1,
        )
    }

    pub fn render(
        &self,
        api: &mut dyn DoryenApi,
        width: i32,
        height: i32,
        row_offset: i32,
        col_offset: i32,
    ) {
        let con = api.con();
        let row_i = height / 2;
        let col_i = width / 2;

        con.ascii(col_i + col_offset, row_i + row_offset, gui::CHAR_PLAYER);
        con.fore(col_i + col_offset, row_i + row_offset, color::WHITE);
    }
}
