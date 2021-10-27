use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub id: String,
    pub zone_row_i: i32,
    pub zone_col_i: i32,
    pub avatar_uuid: Option<String>,
    pub avatar_is_validated: bool,
}

impl Character {
    pub fn position(&self) -> (i32, i32) {
        (self.zone_row_i, self.zone_col_i)
    }
}
