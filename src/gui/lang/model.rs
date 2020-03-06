use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Part {
    pub text: Option<String>,
    pub is_form: bool,
    pub form_action: Option<String>,
    pub form_values_in_query: bool,
    pub items: Vec<Part>,
    pub type_: Option<String>,
    pub label: Option<String>,
    pub name: Option<String>,
    pub is_link: bool,
    pub go_back_zone: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Description {
    pub title: Option<String>,
    pub items: Vec<Part>,
    pub image: Option<String>,
    pub image_id: Option<i32>,
    pub image_extension: Option<String>,
    pub is_long_text: bool,
    pub new_character_id: Option<String>,
}
