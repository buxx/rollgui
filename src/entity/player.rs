use doryen_rs::{DoryenApi};

use crate::color;

#[derive(Debug)]
pub struct Player {
    // row_i, col_i
    pub position: (f32, f32),
    speed: f32,
}

impl Player {
    pub fn new(position: (i32, i32)) -> Self {
        Self {
           position: (position.0 as f32, position.1 as f32),
            speed: 0.2
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
        old_row_i == self.position.0 as i32 && old_col_i == self.position.1 as i32
    }

    pub fn next_position(&self, mov: (i32, i32)) -> (i32, i32) {
        (self.position.0 as i32 + mov.0, self.position.1 as i32 + mov.1)
    }

    pub fn render(&self, api: &mut dyn DoryenApi, width: i32, height: i32) {
        let con = api.con();
        let row_i = height / 2;
        let col_i = width / 2;

        con.ascii(col_i, row_i, '@' as u16);
        con.fore(col_i, row_i, color::WHITE);

    }

}