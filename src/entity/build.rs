use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Build {
    pub id: i32,
    pub build_id: String,
    pub row_i: i32,
    pub col_i: i32,
    pub classes: Vec<String>,
    pub traversable: HashMap<String, bool>,
    pub is_floor: bool,
}

impl Build {
    pub fn position(&self) -> (i32, i32) {
        (self.row_i, self.col_i)
    }

    pub fn get_classes(&self) -> Vec<String> {
        // TODO perf: compute this at object creation
        let mut classes = vec!["BUILD_GENERIC".to_string()];
        classes.extend(self.classes.clone());
        classes
    }
}
