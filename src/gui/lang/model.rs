use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TwoFooterObject {
    back_url: String,
    back_label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ThreeFooterObject {
    back_url: String,
    back_label: String,
    continue_url: String,
    continue_label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Part {
    pub text: Option<String>,
    pub is_form: bool,
    pub form_action: Option<String>,
    pub form_values_in_query: bool,
    pub submit_label: Option<String>,
    pub items: Vec<Part>,
    pub type_: Option<String>,
    pub label: Option<String>,
    pub name: Option<String>,
    pub is_link: bool,
    pub default_value: Option<String>,
    pub link_group_name: Option<String>,
    pub align: Option<String>,
    pub value: Option<String>,
    pub is_checkbox: bool,
    pub checked: bool,
    pub choices: Option<Vec<String>>,
    pub search_by_str: bool,
    pub classes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Description {
    pub origin_url: Option<String>,
    pub title: Option<String>,
    pub items: Vec<Part>,
    pub footer_links: Vec<Part>,
    pub back_url: Option<String>,
    pub back_url_is_zone: bool,
    pub back_to_zone: bool,
    pub image: Option<String>,
    pub image_id: Option<i32>,
    pub image_extension: Option<String>,
    pub is_long_text: bool,
    pub new_character_id: Option<String>,
    pub redirect: Option<String>,
    pub force_back_url: Option<String>,
    pub can_be_back_url: bool,
    pub request_clicks: Option<RequestClicks>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestClicks {
    pub base_url: String,
    pub cursor_classes: Vec<String>,
    pub many: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub message: String,
}
