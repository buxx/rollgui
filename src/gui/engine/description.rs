use doryen_rs::{DoryenApi, Image, UpdateEvent};
use doryen_ui as ui;
use serde_json::{Map, Number, Value};

use crate::gui::action;
use crate::gui::engine::Engine;
use crate::gui::lang::model::Description;
use crate::server::client::ClientError;
use crate::server::Server;
use crate::util;

const UI_WIDTH_MARGIN: i32 = 2;

pub struct DescriptionEngine {
    description: Description,
    server: Server,
    form_error_message: Option<String>,
}

impl DescriptionEngine {
    pub fn new(description: Description, server: Server) -> Self {
        dbg!(&description);
        Self {
            description,
            server,
            form_error_message: None,
        }
    }
}

impl Engine for DescriptionEngine {
    fn get_name(&self) -> &str {
        "DESCRIPTION"
    }

    fn update(
        &mut self,
        _api: &mut dyn DoryenApi,
        _width: i32,
        _height: i32,
    ) -> Option<UpdateEvent> {
        None
    }

    fn render(&mut self, api: &mut dyn DoryenApi, _width: i32, _height: i32) {}

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

        ctx.frame_begin("menu", "frame_title", width, height)
            .margin(0);

        if let Some(title) = &self.description.title {
            for label_line in util::overflow(title.as_str(), width - UI_WIDTH_MARGIN).iter() {
                ctx.label(label_line.as_str()).align(ui::TextAlign::Center);
            }
        }

        let items = &self.description.items;
        for item in items.iter() {
            let mut label: String = "----------".to_string();

            if item.go_back_zone {
                let mut label = "Continuer";
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
            }

            if item.is_link && item.form_action.is_some() {
                let url = item.form_action.as_ref().unwrap();
                for label_line in util::overflow(label.as_str(), width - UI_WIDTH_MARGIN).iter() {
                    if ctx
                        .button("link", label_line.as_str())
                        .align(ui::TextAlign::Left)
                        .pressed()
                    {
                        return Some(action::Action::DescriptionToDescriptionGet {
                            url: url.to_string(),
                        });
                    }
                }
            } else {
                for label_line in util::overflow(label.as_str(), width - UI_WIDTH_MARGIN).iter() {
                    ctx.label(label_line.as_str()).align(ui::TextAlign::Left);
                }
            }

            if item.is_form {
                let form = item;
                let form_action = form.form_action.as_ref().unwrap();
                let mut form_data = Map::new();
                let mut form_query = Map::new();
                let mut form_submit = false;

                if let Some(form_error_message) = &self.form_error_message {
                    ctx.label_color(format!("#[error]{}", form_error_message.as_str()).as_str())
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

                    if let Some(label_) = label {
                        ctx.label(label_.as_str()).align(ui::TextAlign::Left);
                    }
                    if form_item.type_.is_some() && form_item.name.is_some() {
                        ctx.textbox(form_item.name.as_ref().unwrap().as_str(), 32, None, None);
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
                    }

                    if form_item.go_back_zone {
                        let label = item.label.as_deref().unwrap_or("Continuer");
                        form_submit = true;

                        if ctx
                            .button("validate_form", label)
                            .align(ui::TextAlign::Center)
                            .pressed()
                        {
                            if form.form_values_in_query {
                                form_query = form_data.clone();
                                form_data = Map::new();
                            }
                            let description = self.server.client.describe(
                                form_action.as_str(),
                                Some(form_data.clone()),
                                Some(form_query.clone()),
                            );

                            if let Result::Err(client_error) = description {
                                let error_message = ClientError::get_message(&client_error);
                                self.form_error_message = Some(error_message);
                            } else if let Result::Ok(description) = description {
                                return Some(action::Action::DescriptionToDescription {
                                    description,
                                });
                            }
                        }
                    }
                }

                if !form_submit {
                    if ctx
                        .button("validate_form", "Continuer")
                        .align(ui::TextAlign::Center)
                        .pressed()
                    {
                        if form.form_values_in_query {
                            form_query = form_data.clone();
                            form_data = Map::new();
                        }
                        let description = self.server.client.describe(
                            form_action.as_str(),
                            Some(form_data.clone()),
                            Some(form_query.clone()),
                        );

                        if let Result::Err(client_error) = description {
                            let error_message = ClientError::get_message(&client_error);
                            self.form_error_message = Some(error_message);
                        } else if let Result::Ok(description) = description {
                            return Some(action::Action::DescriptionToDescription { description });
                        }
                    }
                }
            }
        }

        None
    }

    fn teardown(&mut self) {}
}
