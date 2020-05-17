use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Build {
    pub id: i32,
    pub build_id: String,
    pub row_i: i32,
    pub col_i: i32,
    pub classes: Vec<String>,
}

impl Build {
    pub fn position(&self) -> (i32, i32) {
        (self.row_i, self.col_i)
    }

    pub fn get_classes(&self) -> Vec<String> {
        // TODO perf: compute this at object creation
        let mut classes = vec!["BUILD_GENERIC".to_string()];
        classes.extend(self.classes.clone());
        classes.push(self.build_id.clone());
        classes
    }
}
