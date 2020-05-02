use doryen_rs::{DoryenApi, Image, TextAlign};
use doryen_ui as ui;
use serde_json::{Map, Number, Value};

use crate::gui::action;
use crate::gui::engine::Engine;
use crate::gui::lang::model::{Description, Part};
use crate::server::client::{Client, ClientError};
use crate::server::Server;
use crate::{color, util};
use std::collections::HashMap;

const UI_WIDTH_MARGIN: i32 = 2;

pub struct DescriptionEngine {
    description: Description,
    server: Server,
    form_error_message: Option<String>,
    start_items_from: i32,
    illustration: Option<Image>,
    illustration_passed: bool,
    loading: bool,
    loading_displayed: bool,
    loading_closure: Option<Box<dyn Fn(Client) -> Result<action::Action, String>>>,
    link_group_name: Option<String>,
    selections: HashMap<String, String>,
    back_url: Option<String>,
}

impl DescriptionEngine {
    pub fn new(description: Description, server: Server, back_url: Option<String>) -> Self {
        Self {
            description,
            server,
            form_error_message: None,
            start_items_from: 0,
            illustration: None,
            illustration_passed: false,
            loading: false,
            loading_displayed: false,
            loading_closure: None,
            link_group_name: None,
            selections: HashMap::new(),
            back_url,
        }
    }
}

impl Engine for DescriptionEngine {
    fn get_name(&self) -> &str {
        "DESCRIPTION"
    }

    fn update(
        &mut self,
        api: &mut dyn DoryenApi,
        _width: i32,
        _height: i32,
    ) -> Option<action::Action> {
        if let Some(redirect) = self.description.redirect.as_ref() {
            return Some(action::Action::DescriptionToDescriptionGet {
                url: redirect.to_string(),
                back_url: None,
            });
        }

        let input = api.input();

        if input.key("ArrowUp") {
            self.start_items_from -= 1;
        } else if input.key("ArrowDown") {
            self.start_items_from += 1;
        }

        if input.key_pressed("Escape") {
            if let Some(force_back_url) = &self.description.force_back_url {
                return Some(action::Action::DescriptionToDescriptionGet {
                    url: force_back_url.clone(),
                    back_url: None,
                });
            }
            if let Some(back_url) = &self.back_url {
                return Some(action::Action::DescriptionToDescriptionGet {
                    url: back_url.clone(),
                    back_url: None,
                });
            }
        }

        if self.start_items_from < 0 {
            self.start_items_from = 0;
        }

        None
    }

    fn render(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) {
        if self.loading {
            api.con()
                .clear(Some(color::BLACK), Some(color::BLACK), Some(' ' as u16));
            api.con().print(
                width / 2,
                height / 2,
                "Chargement ...",
                TextAlign::Center,
                Some(color::WHITE),
                Some(color::BLACK),
            );
            return;
        }

        if let Some(illustration) = self.illustration.as_mut() {
            illustration.blit_2x(api.con(), 0, 0, 0, 0, None, None, None);
        }

        api.con().print_color(
            width - 1,
            0,
            "Echap: Retour",
            TextAlign::Right,
            None,
        );
    }

    fn resize(&mut self, _api: &mut dyn DoryenApi) {}

    fn build_ui(
        &mut self,
        ctx: &mut ui::Context,
        width: i32,
        height: i32,
    ) -> Option<action::Action> {
        if let Some(character_id) = &self.description.new_character_id {
            return Some(action::Action::NewCharacterId {
                character_id: character_id.clone(),
            });
        }
        let future_back_url = if self.description.can_be_back_url {
            self.description.origin_url.clone()
        } else {
            None
        };

        if self.loading {
            if self.loading_displayed {
                match self.loading_closure.as_ref().unwrap()(self.server.client.clone()) {
                    Ok(action) => return Some(action),
                    Err(error_message) => {
                        self.form_error_message = Some(error_message);
                        self.loading = false;
                    }
                }
            }
            self.loading_displayed = true;
            return None;
        }

        ctx.frame_begin("menu", "frame_title", width, height)
            .margin(0);

        let mut displayed_group_names: Vec<String> = Vec::new();
        if let Some(link_group_name) = self.link_group_name.as_ref() {
            for label_line in util::overflow(link_group_name, width - UI_WIDTH_MARGIN).iter() {
                ctx.label(label_line.as_str()).align(ui::TextAlign::Center);
            }
        } else {
            if let Some(title) = &self.description.title {
                for label_line in util::overflow(title.as_str(), width - UI_WIDTH_MARGIN).iter() {
                    ctx.label(label_line.as_str()).align(ui::TextAlign::Center);
                }
            }
        }

        if self.illustration.is_none()
            && !self.illustration_passed
            && self.description.image_id.is_some()
        {
            let image_id = self.description.image_id.unwrap();
            let image_extension = self.description.image_extension.as_ref().unwrap();
            // default dir is "static" !
            let image_path = format!("cache/{}{}", image_id, image_extension);
            if !std::path::Path::new(image_path.as_str()).exists() {
                // TODO: Manage error
                self.server
                    .client
                    .download_image(image_id, image_extension)
                    .unwrap();
            }
            self.illustration = Some(Image::new(image_path.as_str()));
        } else if self.description.image_id.is_some() && !self.illustration_passed {
            if ctx
                .button("continue", "Continuer")
                .align(ui::TextAlign::Center)
                .pressed()
            {
                self.illustration_passed = true;
                self.illustration = None;
            }
        } else {
            let mut items = self.description.items.clone();
            if items.len() == 0 {
                items.push(Part {
                    ..Default::default()
                });
            }

            // if group name displayed, filter these
            if let Some(link_group_name) = self.link_group_name.as_ref() {
                items.retain(|item| item.link_group_name == Some(link_group_name.to_string()));
            }

            if self.start_items_from as usize >= items.len() {
                self.start_items_from = items.len() as i32 - 1;
            }
            let items = &items[self.start_items_from as usize..];
            for item in items.iter() {
                let mut label: String = "----------".to_string();

                if item.go_back_zone {
                    let label = item.label.as_deref().unwrap_or("Continuer");
                    if ctx
                        .button("validate_form", label)
                        .align(ui::TextAlign::Center)
                        .pressed()
                    {
                        return Some(action::Action::DescriptionToZone);
                    }
                } else {
                    if item.text.is_some() && item.label.is_some() {
                        let label_ = item.label.as_ref().unwrap();
                        let text = item.text.as_ref().unwrap();
                        label = format!("{}: {}", label_, text);
                    } else {
                        if let Some(text) = &item.text {
                            label = text.to_string();
                        }
                        if let Some(text) = &item.label {
                            label = text.to_string();
                        }
                    }

                    let mut exclude_item = false;
                    if self.link_group_name.is_none() {
                        if let Some(link_group_name) = item.link_group_name.as_ref() {
                            if displayed_group_names.contains(link_group_name) {
                                exclude_item = true;
                            } else {
                                displayed_group_names.push(link_group_name.clone());
                            }
                        }
                    }

                    if !exclude_item && item.is_link && item.form_action.is_some() {
                        let url = item.form_action.as_ref().unwrap();
                        let label = if self.link_group_name.is_none() {
                            item.link_group_name.as_deref().unwrap_or(label.as_str())
                        } else {
                            label.as_str()
                        };
                        for label_line in util::overflow(label, width - UI_WIDTH_MARGIN).iter() {
                            let align = match item.align.as_ref() {
                                Some(align) => match align.as_str() {
                                    "left" => ui::TextAlign::Left,
                                    "center" => ui::TextAlign::Center,
                                    "right" => ui::TextAlign::Right,
                                    _ => ui::TextAlign::Center,
                                },
                                _ => ui::TextAlign::Center,
                            };
                            if ctx
                                .button(label_line.as_str(), label_line.as_str())
                                .align(align)
                                .pressed()
                            {
                                if self.link_group_name.is_none() && item.link_group_name.is_some()
                                {
                                    self.link_group_name = item.link_group_name.clone();
                                } else {
                                    return Some(action::Action::DescriptionToDescriptionGet {
                                        url: url.to_string(),
                                        back_url: future_back_url.clone(),
                                    });
                                }
                            }
                        }
                    } else if !exclude_item {
                        for line in label.split("\n") {
                            for label_line in util::overflow(line, width - UI_WIDTH_MARGIN).iter() {
                                ctx.label(label_line.as_str()).align(ui::TextAlign::Left);
                            }
                        }
                    }
                }

                if item.is_form {
                    let form = item;
                    let form_action = form.form_action.as_ref().unwrap();
                    let mut form_data = Map::new();
                    let form_query = Map::new();
                    let mut form_submit_label = item
                        .submit_label
                        .as_ref()
                        .unwrap_or(&"Continuer".to_string())
                        .clone();

                    if let Some(form_error_message) = &self.form_error_message {
                        ctx.label_color(
                            format!("#[error]{}", form_error_message.as_str()).as_str(),
                        )
                        .align(ui::TextAlign::Left);
                    }

                    for form_item in form.items.iter() {
                        let mut label: Option<String> = None;
                        if let Some(text) = &form_item.text {
                            label = Some(text.to_string());
                        }
                        if let Some(text) = &form_item.label {
                            label = Some(text.to_string());
                        }
                        if form_item.type_.is_some() && form_item.name.is_some() {
                            label = Some(format!("{}:", label.unwrap()));
                        }

                        if !form_item.is_link {
                            if let Some(label_) = label.clone() {
                                for label_line in
                                    util::overflow(&label_, width - UI_WIDTH_MARGIN).iter()
                                {
                                    ctx.label(label_line.as_str()).align(ui::TextAlign::Left);
                                }
                            }
                        }

                        if form_item.is_link {
                            if let Some(link_label) = label {
                                if ctx
                                    .button(link_label.as_str(), link_label.as_str())
                                    .align(ui::TextAlign::Center)
                                    .pressed()
                                {
                                    return Some(action::Action::DescriptionToDescriptionGet {
                                        url: form_item.form_action.as_ref().unwrap().clone(),
                                        back_url: future_back_url.clone(),
                                    });
                                }
                            }
                        } else if form_item.type_.is_some() && form_item.name.is_some() {
                            let default_value = form_item.default_value.as_deref().unwrap_or("");
                            ctx.textbox(
                                form_item.name.as_ref().unwrap().as_str(),
                                32,
                                Some(&default_value),
                                None,
                            );
                            let input_name = form_item.name.as_ref().unwrap().clone();
                            let value = ctx.text(ctx.last_id());
                            match form_item.type_.as_ref().unwrap().as_str() {
                                "STRING" | "TEXT" => {
                                    form_data.insert(input_name, Value::String(value.to_string()));
                                }
                                "NUMBER" => {
                                    if value == "" {
                                        form_data.insert(
                                            input_name,
                                            Value::Number(Number::from_f64(0f64).unwrap()),
                                        );
                                    } else {
                                        if let Ok(float_) = value.parse::<f64>() {
                                            form_data.insert(
                                                input_name,
                                                Value::Number(Number::from_f64(float_).unwrap()),
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                        } else if form_item.is_checkbox {
                            let label = form_item.label.as_ref().unwrap();
                            if ctx.checkbox(label, label, form_item.checked).active() {
                                let key = form_item.name.as_ref().unwrap().clone();
                                let value =
                                    Value::String(form_item.value.as_ref().unwrap().clone());
                                form_data.insert(key, value);
                            }
                        } else if form_item.choices.is_some() && !form_item.search_by_str {
                            let key = form_item.name.as_ref().unwrap().clone();
                            let default_value =
                                form_item.value.as_ref().unwrap_or(&"".to_string()).clone();
                            let default_position = form_item
                                .choices
                                .as_ref()
                                .unwrap()
                                .iter()
                                .position(|x| x.eq(&default_value))
                                .unwrap();
                            ctx.list_button_begin(&key, default_position as i32);
                            for choice in form_item.choices.as_ref().unwrap().iter() {
                                if ctx.list_button_item(choice, ui::TextAlign::Center) {
                                    let value = Value::String(choice.to_string());
                                    form_data.insert(key.clone(), value);
                                }
                            }
                            ctx.list_button_end(true);
                        } else if form_item.choices.is_some() && form_item.search_by_str {
                            let input_name = form_item.name.as_ref().unwrap().as_str();
                            ctx.textbox(input_name, 32, None, Some("Chercher"));
                            let textbox_id = ctx.last_id();
                            let value: &str = ctx.text(ctx.last_id());
                            let mut choices: Vec<String> =
                                form_item.choices.as_ref().unwrap().clone();
                            if !value.is_empty() {
                                choices = choices
                                    .into_iter()
                                    .filter(|string_| {
                                        string_
                                            .to_lowercase()
                                            .matches(value.to_lowercase().as_str())
                                            .collect::<String>()
                                            .len()
                                            != 0
                                    })
                                    .collect();
                            }
                            for choice in choices.iter() {
                                let label = if self.selections.get(choice).is_some() {
                                    format!("*{}*", choice)
                                } else {
                                    choice.to_string()
                                };

                                if ctx
                                    .button(choice.as_str(), label.as_str())
                                    .align(ui::TextAlign::Center)
                                    .pressed()
                                {
                                    ctx.set_textbox_value(textbox_id, choice);
                                    self.selections
                                        .insert(input_name.to_string(), choice.to_string());
                                }
                            }
                            if let Some(value_) = self.selections.get(input_name) {
                                let value = Value::String(value_.to_string());
                                form_data.insert(input_name.to_string(), value);
                            }
                        }

                        if form_item.go_back_zone {
                            form_submit_label = item
                                .label
                                .as_ref()
                                .unwrap_or(&"Continuer".to_string())
                                .clone();
                        }
                    }

                    if ctx
                        .button("validate_form", &form_submit_label)
                        .align(ui::TextAlign::Center)
                        .pressed()
                    {
                        let press_form_action = form_action.clone();
                        let press_form_query = if form.form_values_in_query {
                            form_data.clone()
                        } else {
                            form_query.clone()
                        };
                        let press_form_data = if form.form_values_in_query {
                            Map::new()
                        } else {
                            form_data.clone()
                        };
                        let next_back_url = future_back_url.clone();
                        self.loading = true;
                        self.loading_closure = Some(Box::new(move |client| {
                            let description = client.describe(
                                press_form_action.as_str(),
                                Some(press_form_data.clone()),
                                Some(press_form_query.clone()),
                            );
                            return match description {
                                Result::Err(client_error) => {
                                    Err(ClientError::get_message(&client_error))
                                }
                                Result::Ok(description) => {
                                    Ok(action::Action::DescriptionToDescription {
                                        description,
                                        back_url: next_back_url.clone(),
                                    })
                                }
                            };
                        }));
                    }
                }
            }

            if self.link_group_name.is_some() {
                if ctx
                    .button("go_back_without_link_group_name", "Retour")
                    .align(ui::TextAlign::Center)
                    .pressed()
                {
                    self.link_group_name = None;
                }
            }
        }

        None
    }

    fn teardown(&mut self) {}
}
