use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimatedCorpse {
    pub id: i32,
    pub type_: String,
    pub zone_row_i: i32,
    pub zone_col_i: i32,
}

impl AnimatedCorpse {
    pub fn position(&self) -> (i32, i32) {
        (self.zone_row_i, self.zone_col_i)
    }
}
