use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Build {
    pub id: i32,
    pub row_i: i32,
    pub col_i: i32,
}

impl Build {
    pub fn position(&self) -> (i32, i32) {
        (self.row_i, self.col_i)
    }
}
