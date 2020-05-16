use crate::game::{TILE_HEIGHT, TILE_WIDTH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: String,
    pub x: i16,
    pub y: i16,
    // row_i, col_i
    pub position: (i32, i32),
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
    pub unread_conversation: bool,
    pub unvote_affinity_relation: bool,
    pub unread_transactions: bool,
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
    pub unread_conversation: bool,
    pub unvote_affinity_relation: bool,
    pub unread_transactions: bool,
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
        unread_conversation: bool,
        unvote_affinity_relation: bool,
        unread_transactions: bool,
    ) -> Self {
        Self {
            id: id.to_string(),
            // TODO: tile width/height must be passed by args
            x: position.1 as i16 * TILE_WIDTH,
            y: position.0 as i16 * TILE_HEIGHT,
            position: (position.0, position.1),
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
            unread_conversation,
            unvote_affinity_relation,
            unread_transactions,
        }
    }

    pub fn try_move_by(&mut self, x: i16, y: i16) -> (bool, bool) {
        let before_x = self.x;
        let before_y = self.y;
        let before_row_i = self.position.0;
        let before_col_i = self.position.1;

        if (self.x + x) > -1 {
            self.x += x;
        }
        if (self.y + y) > -1 {
            self.y += y;
        }

        self.position.0 = (self.y / TILE_HEIGHT) as i32;
        self.position.1 = (self.x / TILE_WIDTH) as i32;
        (
            before_x != x || before_y != y,
            before_row_i != self.position.0 || before_col_i != self.position.1,
        )
    }

    pub fn next_position(&self, x: i16, y: i16) -> Option<(i16, i16)> {
        let before_row_i = self.position.0;
        let before_col_i = self.position.1;
        let mut next_x = self.x + x;
        let mut next_y = self.y + y;

        if x > 0 {
            next_x += TILE_WIDTH / 2;
        }
        if y > 0 {
            next_y += TILE_HEIGHT / 2;
        }
        let next_row_i_f = next_y as f32 / TILE_HEIGHT as f32;
        let next_col_i_f = next_x as f32 / TILE_WIDTH as f32;

        let next_row_i = if next_row_i_f < 0.0 {
            -1
        } else {
            next_y / TILE_HEIGHT
        };

        let next_col_i = if next_col_i_f < 0.0 {
            -1
        } else {
            next_x / TILE_WIDTH
        };

        if before_row_i != next_row_i as i32 || before_col_i != next_col_i as i32 {
            return Some((next_row_i, next_col_i));
        }
        None
    }
}
