use crate::engine::Engine;
use crate::gui::lang::model::{Description, Part};
use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use crate::server::client::ClientError;
use crate::server::Server;
use crate::ui::widget::button::Button;
use crate::ui::widget::checkbox::Checkbox;
use crate::ui::widget::radio::Radio;
use crate::ui::widget::state_less_button::StateLessButton;
use crate::ui::widget::text::Text;
use crate::ui::widget::text_input::TextInput;
use crate::ui::widget::{button, state_less_button};
use crate::ui::Element;
use crate::ui::{Column, Row};
use coffee::graphics::{Color, Frame, HorizontalAlignment, Point, VerticalAlignment, Window};
use coffee::input::keyboard;
use coffee::ui::{Align, Justify};
use coffee::Timer;
use serde_json::{Map, Number, Value};
use std::cmp::max;
use std::collections::HashMap;
use std::time::Instant;

const BLINK_MS: u128 = 250;

pub struct DescriptionEngine {
    description: Description,
    server: Server,
    error_message: Option<String>,
    text_input_selected: i32,
    text_input_ids: HashMap<String, i32>,
    text_input_names: HashMap<i32, String>,
    text_input_values: HashMap<i32, String>,
    text_input_types: HashMap<i32, String>,
    link_button_ids: HashMap<String, i32>,
    link_button_pressed: i32,
    blink_time: Instant,
    submit_button: button::State,
    go_back_zone_button: button::State,
    current_link_group_name: Option<String>,
    link_group_name_ids: HashMap<String, i32>,
    link_group_button_pressed: i32,
    back_from_group_by_button: button::State,
    checkbox_ids: HashMap<String, i32>,
    checkbox_names: HashMap<i32, String>,
    checkbox_values: HashMap<i32, String>,
    back_url: Option<String>,
    future_back_url: Option<String>,
    choice_ids: HashMap<String, i32>,
    choice_names: HashMap<i32, String>,
    choice_values: HashMap<i32, String>,
    choice_values_ids: HashMap<String, i32>,
    choice_values_values: HashMap<i32, String>,
    search_by_str_ids: HashMap<String, i32>,
    search_by_str_names: HashMap<i32, String>,
    search_by_str_values: HashMap<i32, String>,
    search_by_str_button_ids: HashMap<String, i32>,
    search_by_str_button_values: HashMap<i32, String>,
    search_by_str_selected: i32,
    search_by_str_button_pressed: i32,
    pending_request: Option<(String, Map<String, Value>, Map<String, Value>)>,
    loading_displayed: bool,
    force_back_startup: bool,
    start_items_from: i32,
    submitable: bool,
}

impl DescriptionEngine {
    pub fn new(
        description: Description,
        server: Server,
        back_url: Option<String>,
        force_back_startup: bool,
    ) -> Self {
        let mut link_button_ids = HashMap::new();
        let mut text_input_ids = HashMap::new();
        let mut text_input_names = HashMap::new();
        let mut text_input_types = HashMap::new();
        let mut text_input_values = HashMap::new();
        let mut text_input_selected: i32 = -1;
        let mut link_group_name_ids = HashMap::new();
        let mut checkbox_values = HashMap::new();
        let mut checkbox_ids = HashMap::new();
        let mut checkbox_names = HashMap::new();
        let mut choice_values = HashMap::new();
        let mut choice_ids = HashMap::new();
        let mut choice_names = HashMap::new();
        let mut choice_values_ids = HashMap::new();
        let mut choice_values_values = HashMap::new();
        let mut search_by_str_ids = HashMap::new();
        let mut search_by_str_values = HashMap::new();
        let mut search_by_str_button_ids = HashMap::new();
        let mut search_by_str_button_values = HashMap::new();
        let mut search_by_str_names = HashMap::new();
        let mut text_input_counter: i32 = 0;
        let mut link_button_counter: i32 = 0;
        let mut checkbox_counter: i32 = 0;
        let mut choice_counter: i32 = 0;
        let mut choice_values_counter: i32 = 0;
        let mut search_by_str_counter: i32 = 0;
        let mut search_by_str_button_counter: i32 = 0;
        let mut search_by_str_selected = -1;
        let mut submitable = false;

        for item in description.items.iter() {
            if part_is_form(item) {
                submitable = true;
                for form_item in item.items.iter() {
                    if part_is_input(form_item) {
                        text_input_ids
                            .insert(form_item.name.as_ref().unwrap().clone(), text_input_counter);
                        text_input_values.insert(
                            text_input_counter,
                            form_item
                                .default_value
                                .as_ref()
                                .unwrap_or(&"".to_string())
                                .clone(),
                        );
                        text_input_names
                            .insert(text_input_counter, form_item.name.as_ref().unwrap().clone());
                        text_input_types.insert(
                            text_input_counter,
                            form_item.type_.as_ref().unwrap().clone(),
                        );

                        if text_input_selected == -1 && search_by_str_selected == -1 {
                            text_input_selected = text_input_counter;
                        }

                        text_input_counter += 1;
                    } else if part_is_checkbox(form_item) {
                        checkbox_ids
                            .insert(form_item.name.as_ref().unwrap().clone(), checkbox_counter);
                        checkbox_names
                            .insert(checkbox_counter, form_item.name.as_ref().unwrap().clone());
                        if form_item.checked {
                            checkbox_values.insert(checkbox_counter, "on".to_string());
                        }
                        checkbox_counter += 1;
                    } else if part_is_link(form_item) {
                        let label = form_item
                            .label
                            .as_ref()
                            .unwrap_or(form_item.text.as_ref().unwrap_or(&" ".to_string()))
                            .clone();
                        link_button_ids.insert(label, link_button_counter);
                        link_button_counter += 1;
                    } else if part_is_choices(form_item) {
                        choice_ids.insert(form_item.name.as_ref().unwrap().clone(), choice_counter);
                        choice_names
                            .insert(choice_counter, form_item.name.as_ref().unwrap().clone());
                        choice_values
                            .insert(choice_counter, form_item.value.as_ref().unwrap().clone());

                        for choice in form_item.choices.as_ref().unwrap().iter() {
                            choice_values_ids.insert(choice.clone(), choice_values_counter);
                            choice_values_values.insert(choice_values_counter, choice.clone());
                            choice_values_counter += 1;
                        }

                        choice_counter += 1;
                    } else if part_is_search_by_str(form_item) {
                        search_by_str_ids.insert(
                            form_item.name.as_ref().unwrap().clone(),
                            search_by_str_counter,
                        );
                        search_by_str_values.insert(search_by_str_counter, "".to_string());
                        search_by_str_names.insert(
                            search_by_str_counter,
                            form_item.name.as_ref().unwrap().clone(),
                        );

                        for choice in form_item.choices.as_ref().unwrap().iter() {
                            search_by_str_button_ids
                                .insert(choice.clone(), search_by_str_button_counter);
                            search_by_str_button_values
                                .insert(search_by_str_button_counter, choice.clone());
                            search_by_str_button_counter += 1;
                        }

                        if search_by_str_selected == -1 && text_input_selected == -1 {
                            search_by_str_selected = search_by_str_counter;
                        }

                        search_by_str_counter += 1;
                    }
                }
            } else if part_is_link(item) {
                link_button_ids.insert(
                    item.label
                        .as_ref()
                        .unwrap_or(item.text.as_ref().unwrap_or(&"Continuer".to_string()))
                        .clone(),
                    link_button_counter,
                );

                // Assume link group names are not in forms
                if let Some(link_group_name) = item.link_group_name.as_ref() {
                    link_group_name_ids.insert(link_group_name.clone(), link_button_counter);
                }
                link_button_counter += 1;
            }
        }

        for link_item in description.footer_links.iter() {
            let label = link_item
                .label
                .as_ref()
                .unwrap_or(link_item.text.as_ref().unwrap_or(&" ".to_string()))
                .clone();
            link_button_ids.insert(label, link_button_counter);
            link_button_counter += 1;
        }

        let future_back_url = if description.can_be_back_url {
            description.origin_url.clone()
        } else {
            None
        };

        Self {
            description,
            server,
            error_message: None,
            text_input_selected,
            text_input_ids,
            text_input_types,
            text_input_names,
            text_input_values,
            link_button_ids,
            link_button_pressed: -1,
            blink_time: Instant::now(),
            submit_button: button::State::new(),
            go_back_zone_button: button::State::new(),
            current_link_group_name: None,
            link_group_name_ids,
            link_group_button_pressed: -1,
            back_from_group_by_button: button::State::new(),
            checkbox_ids,
            checkbox_names,
            checkbox_values,
            // form_error_message: None,
            // start_items_from: 0,
            // illustration: None,
            // illustration_passed: false,
            // loading: false,
            // loading_displayed: false,
            // loading_closure: None,
            // link_group_name: None,
            // selections: HashMap::new(),
            back_url,
            future_back_url,
            choice_ids,
            choice_names,
            choice_values,
            choice_values_ids,
            choice_values_values,
            search_by_str_ids,
            search_by_str_values,
            search_by_str_button_ids,
            search_by_str_button_values,
            search_by_str_selected,
            search_by_str_button_pressed: -1,
            search_by_str_names,
            pending_request: None,
            loading_displayed: false,
            force_back_startup,
            start_items_from: 0,
            submitable,
        }
    }

    fn apply_text_buffer(&mut self, text_input_id: i32, text_buffer: String) {
        if let Some(text_input_value) = self.text_input_values.get_mut(&text_input_id) {
            for c in text_buffer.chars() {
                match c {
                    // Match ASCII backspace and delete from the text buffer
                    '\u{8}' => {
                        text_input_value.pop();
                    }
                    // Tabulation | Enter | Escape
                    '\t' | '\r' | '\u{1b}' => {}
                    _ => {
                        text_input_value.push(c);
                    }
                }
            }
        }
    }

    fn apply_search_by_str_buffer(&mut self, search_by_str_id: i32, text_buffer: String) {
        if let Some(search_by_str_value) = self.search_by_str_values.get_mut(&search_by_str_id) {
            for c in text_buffer.chars() {
                match c {
                    // Match ASCII backspace and delete from the text buffer
                    '\u{8}' => {
                        search_by_str_value.pop();
                    }
                    // Tabulation | Enter | Escape
                    '\t' | '\r' | '\u{1b}' => {}
                    _ => {
                        search_by_str_value.push(c);
                    }
                }
            }
        }
    }

    fn get_blink_char(&mut self) -> Option<char> {
        let elapsed = self.blink_time.elapsed().as_millis();
        if elapsed < BLINK_MS as u128 {
            None
        } else if elapsed <= (BLINK_MS * 2) as u128 {
            Some('_')
        } else {
            self.blink_time = Instant::now();
            None
        }
    }

    fn get_form_data(&self) -> Map<String, Value> {
        let mut data = Map::new();

        for (id, value) in self.text_input_values.iter() {
            let name = self.text_input_names.get(id).unwrap();
            let typed_value = match self.text_input_types.get(id).unwrap().as_ref() {
                "STRING" | "TEXT" => Value::String(value.clone()),
                "NUMBER" => {
                    if value == "" {
                        Value::Number(Number::from_f64(0f64).unwrap())
                    } else {
                        if let Ok(float_) = value.parse::<f64>() {
                            Value::Number(Number::from_f64(float_).unwrap())
                        } else {
                            Value::String(value.to_string())
                        }
                    }
                }
                _ => Value::String(value.to_string()),
            };
            data.insert(name.clone(), typed_value);
        }

        for (id, _) in self.checkbox_values.iter() {
            let name = self.checkbox_names.get(id).unwrap();
            data.insert(name.clone(), Value::String("on".to_string()));
        }

        for (radio_id, radio_value) in self.choice_values.iter() {
            let radio_name = self.choice_names.get(radio_id).unwrap();
            data.insert(radio_name.clone(), Value::String(radio_value.clone()));
        }

        for (id, value) in self.search_by_str_values.iter() {
            let input_name = self.search_by_str_names.get(id).unwrap();
            data.insert(input_name.clone(), Value::String(value.clone()));
        }

        data
    }

    fn form_data_in_query(&self) -> bool {
        // Allow only one form in description !
        for item in self.description.items.iter() {
            if part_is_form(item) {
                return item.form_values_in_query;
            }
        }
        false
    }

    fn get_form_action(&self) -> Option<String> {
        // Allow only one form in description !
        for item in self.description.items.iter() {
            if part_is_form(item) {
                return item.form_action.as_ref().cloned();
            }
        }
        None
    }

    fn submit_form(&mut self) {
        let form_data = self.get_form_data();
        let force_in_query = self.form_data_in_query();
        let form_action = self.get_form_action().unwrap();
        self.error_message = None;

        let (final_form_query, final_form_data) = if force_in_query {
            (form_data.clone(), Map::new())
        } else {
            (Map::new(), form_data.clone())
        };

        self.pending_request = Some((form_action, final_form_data, final_form_query));
    }
}

fn part_is_pure_text(part: &Part) -> bool {
    part.text.is_some() && !part.is_link
}

fn get_part_pure_text_text(part: &Part) -> String {
    if let Some(label) = part.label.as_ref() {
        if let Some(text) = part.text.as_ref() {
            return format!("{}: {}", label, text);
        }
        return label.clone();
    }

    return part.text.as_ref().unwrap().clone();
}

fn part_is_form(part: &Part) -> bool {
    part.is_form
}

fn part_is_input(part: &Part) -> bool {
    part.name.is_some() && part.type_.is_some()
}

fn part_is_link(part: &Part) -> bool {
    part.is_link && part.form_action.is_some()
}

fn part_is_checkbox(part: &Part) -> bool {
    part.is_checkbox
}

fn part_is_choices(part: &Part) -> bool {
    part.choices.is_some() && !part.search_by_str
}

fn part_is_search_by_str(part: &Part) -> bool {
    part.choices.is_some() && part.search_by_str
}

fn part_is_go_back_to_zone(part: &Part) -> bool {
    part.go_back_zone
}

fn default_text_style(text: Text) -> Text {
    text.size(20).vertical_alignment(VerticalAlignment::Center)
}

impl Engine for DescriptionEngine {
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
    }

    fn update(&mut self, _window: &Window) -> Option<MainMessage> {
        if let Some(redirect) = self.description.redirect.as_ref() {
            return Some(MainMessage::ToDescriptionWithUrl {
                url: redirect.to_string(),
                back_url: None,
            });
        }

        if self.loading_displayed {
            let (url, form_data, form_query) = self.pending_request.as_ref().unwrap().clone();
            self.pending_request = None;
            self.loading_displayed = false;

            let try_description = self.server.client.describe(
                url.as_str(),
                Some(form_data.clone()),
                Some(form_query.clone()),
            );
            match try_description {
                Result::Err(client_error) => {
                    self.error_message = Some(ClientError::get_message(&client_error));
                }
                Result::Ok(description) => {
                    if let Some(character_id) = description.new_character_id {
                        return Some(MainMessage::NewCharacterId {
                            character_id: character_id.clone(),
                        });
                    }
                    return Some(MainMessage::ToDescriptionWithDescription {
                        description,
                        back_url: self.future_back_url.clone(),
                    });
                }
            }
        }

        None
    }

    fn interact(&mut self, input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        if !input.text_buffer.is_empty() {
            if self.text_input_selected != -1 {
                self.apply_text_buffer(self.text_input_selected, input.text_buffer.clone());
            } else if self.search_by_str_selected != -1 {
                self.apply_search_by_str_buffer(
                    self.search_by_str_selected,
                    input.text_buffer.clone(),
                );
            }
            input.text_buffer = String::new();
        }

        match input.key_code {
            Some(keyboard::KeyCode::Escape) => {
                input.key_code = None;
                if self.force_back_startup {
                    return Some(MainMessage::ToStartup);
                }
                if let Some(force_back_url) = &self.description.force_back_url {
                    return Some(MainMessage::ToDescriptionWithUrl {
                        url: force_back_url.clone(),
                        back_url: None,
                    });
                }
                if let Some(back_url) = &self.back_url {
                    return Some(MainMessage::ToDescriptionWithUrl {
                        url: back_url.clone(),
                        back_url: None,
                    });
                }
                return Some(MainMessage::DescriptionToZone);
            }
            Some(keyboard::KeyCode::Return) => {
                input.key_code = None;
                if self.submitable {
                    self.submit_form();
                }
            }
            Some(keyboard::KeyCode::Tab) => {
                if self
                    .text_input_names
                    .contains_key(&(self.text_input_selected + 1))
                {
                    self.text_input_selected += 1;
                }
            }
            _ => {}
        }

        if input.keys_pressed.contains(&keyboard::KeyCode::Up) {
            self.start_items_from = max(0, self.start_items_from - 1);
        }

        if input.keys_pressed.contains(&keyboard::KeyCode::Down) {
            self.start_items_from += 1;
        }

        self.start_items_from = max(
            0,
            self.start_items_from - input.mouse_wheel.y.round() as i32,
        );
        input.mouse_wheel = Point::new(0.0, 0.0);

        None
    }

    fn react(&mut self, event: Message, _window: &mut Window) -> Option<MainMessage> {
        match event {
            Message::TextInputSelected(id) => {
                self.text_input_selected = id;
                self.search_by_str_selected = -1;
            }
            Message::SearchByStrInputPressed(id) => {
                self.search_by_str_selected = id;
                self.text_input_selected = -1;
            }
            Message::SearchByStrButtonPressed(_, value_id) => {
                self.search_by_str_button_pressed = value_id;
            }
            Message::SearchByStrButtonReleased(id, value_id) => {
                self.search_by_str_button_pressed = -1;
                self.search_by_str_values.insert(
                    id,
                    self.search_by_str_button_values
                        .get(&value_id)
                        .unwrap()
                        .clone(),
                );
            }
            Message::SubmitButtonPressed => {
                self.submit_form();
            }
            Message::LinkButtonPressed(id) => {
                self.link_button_pressed = id;
            }
            Message::GroupLinkButtonPressed(id) => {
                self.link_group_button_pressed = id;
            }
            Message::LinkButtonReleased(url) => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: url.clone(),
                    back_url: self.future_back_url.clone(),
                });
            }
            Message::GroupLinkButtonReleased(label) => {
                self.current_link_group_name = Some(label.clone());
            }
            Message::GoBackZoneButtonPressed => return Some(MainMessage::DescriptionToZone),
            Message::GoBackFromGroupButtonPressed => {
                self.current_link_group_name = None;
            }
            Message::CheckBoxChecked(id) => {
                self.checkbox_values.insert(id, "on".to_string());
            }
            Message::CheckBoxUnchecked(id) => {
                self.checkbox_values.remove(&id);
            }
            Message::ChoicePressed(radio_id, value_id) => {
                let value = self.choice_values_values.get(&value_id).unwrap().clone();
                self.choice_values.insert(radio_id, value);
            }
            _ => {}
        }

        None
    }

    fn layout(&mut self, window: &Window) -> Element {
        if self.pending_request.is_some() {
            self.loading_displayed = true;
            Column::new()
                .width(window.width() as u32)
                .height(window.height() as u32)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .spacing(20)
                .push(
                    Text::new("Chargement ...")
                        .size(50)
                        .height(60)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .into()
        } else {
            let description = self.description.clone();
            let title = description
                .title
                .as_ref()
                .unwrap_or(&"Sans titre".to_string())
                .clone();
            let items = description.items;

            if self.start_items_from as usize >= items.len() {
                self.start_items_from = items.len() as i32 - 1;
            }
            let items = if items.len() != 0 {
                &items[self.start_items_from as usize..]
            } else {
                &items
            };

            let blink_char = self.get_blink_char();
            let mut must_add_submit = false;
            let mut submit_label = String::from("Enregistrer");
            let text_input_ids = self.text_input_ids.clone();
            let text_input_values = self.text_input_values.clone();
            let text_input_selected = self.text_input_selected.clone();
            let mut back_to_zone_button: Option<String> = None;
            let mut replaced_by_group_names: Vec<String> = vec![];
            let mut ignore_checkbox_ids: Vec<i32> = vec![];

            let mut content = Column::new()
                .max_width(768)
                .spacing(5)
                .push(Text::new(&title).size(50));

            if let Some(error_message) = self.error_message.as_ref() {
                content = content.push(Text::new(error_message).color(Color::RED));
            }

            for item in items.iter() {
                if part_is_pure_text(item) {
                    content = content.push(default_text_style(Text::new(
                        &get_part_pure_text_text(item),
                    )));
                    continue;
                }

                if part_is_form(&item) {
                    must_add_submit = true;
                    if let Some(submit_label_) = item.submit_label.as_ref() {
                        submit_label = submit_label_.clone()
                    }

                    for form_item in item.items.iter() {
                        let label = form_item
                            .label
                            .as_ref()
                            .unwrap_or(form_item.text.as_ref().unwrap_or(&"".to_string()))
                            .clone();

                        if part_is_pure_text(form_item) {
                            content = content.push(default_text_style(Text::new(
                                &get_part_pure_text_text(form_item),
                            )));
                        } else if part_is_input(form_item) {
                            let form_item_name = form_item.name.as_ref().unwrap().clone();
                            let form_item_id = text_input_ids.get(&form_item_name).unwrap();
                            content = content.push(TextInput::new(
                                *form_item_id,
                                &label,
                                text_input_values.get(&form_item_id).unwrap(),
                                Message::TextInputSelected,
                                if text_input_selected == *form_item_id {
                                    blink_char
                                } else {
                                    None
                                },
                            ));
                        } else if part_is_checkbox(form_item) {
                            let name = form_item.name.as_ref().unwrap().clone();
                            let id = self.checkbox_ids.get(&name).unwrap().clone();

                            if !ignore_checkbox_ids.contains(&id) {
                                let mut column1 = Column::new();
                                let mut column2 = Column::new();

                                let mut started = false;
                                let mut counter = 0;
                                for form_item_ in item.items.iter() {
                                    if part_is_checkbox(form_item_) {
                                        let name_ = form_item_.name.as_ref().unwrap().clone();
                                        let id_ = self.checkbox_ids.get(&name_).unwrap().clone();
                                        let label_ = form_item_
                                            .label
                                            .as_ref()
                                            .unwrap_or(
                                                form_item_.text.as_ref().unwrap_or(&"".to_string()),
                                            )
                                            .clone();

                                        if id_ == id {
                                            started = true;
                                        }

                                        if started && !ignore_checkbox_ids.contains(&id_) {
                                            let checkbox = Checkbox::new(
                                                self.checkbox_values.get(&id_).is_some(),
                                                &label_,
                                                move |c| {
                                                    if c {
                                                        Message::CheckBoxChecked(id_)
                                                    } else {
                                                        Message::CheckBoxUnchecked(id_)
                                                    }
                                                },
                                            );

                                            if (counter % 2) == 0 {
                                                column1 = column1.push(checkbox);
                                            } else {
                                                column2 = column2.push(checkbox);
                                            }

                                            counter += 1;
                                            ignore_checkbox_ids.push(id_);
                                        }
                                    } else {
                                        if started {
                                            content = content
                                                .push(Row::new().push(column1).push(column2));
                                            break;
                                        }
                                    }
                                }
                            }
                        } else if part_is_link(form_item) {
                            let label =
                                form_item.label.as_ref().unwrap_or(&" ".to_string()).clone();
                            let display_label = if item.text.is_some() && item.label.is_some() {
                                format!(
                                    "{}: {}",
                                    item.label.as_ref().unwrap().clone(),
                                    item.text.as_ref().unwrap().clone()
                                )
                            } else {
                                label.clone()
                            };
                            let id = *self.link_button_ids.get(&label).unwrap();
                            content = content.push(
                                StateLessButton::new(
                                    self.link_button_pressed == id,
                                    &display_label,
                                    Message::LinkButtonPressed(id),
                                    Message::LinkButtonReleased(
                                        form_item.form_action.as_ref().unwrap().clone(),
                                    ),
                                )
                                .width(768)
                                .class(state_less_button::Class::Primary),
                            );
                        } else if part_is_choices(form_item) {
                            let radio_id = *self
                                .choice_ids
                                .get(form_item.name.as_ref().unwrap())
                                .unwrap();

                            let choices = form_item.choices.as_ref().unwrap();
                            let count_by_column = (choices.len() as f32 / 2.0).ceil() as i32;
                            let mut column1 = Column::new();
                            let mut column2 = Column::new();
                            let mut choices_chunks = choices.chunks(count_by_column as usize);
                            let chunk1 = choices_chunks.next().unwrap();
                            let chunk2 = choices_chunks.next().unwrap();

                            for choice in chunk1 {
                                let value_id = self.choice_values_ids.get(choice).unwrap();
                                column1 = column1.push(Radio::new(
                                    value_id,
                                    choice,
                                    self.choice_values_ids
                                        .get(self.choice_values.get(&radio_id).unwrap()),
                                    move |value_id| Message::ChoicePressed(radio_id, *value_id),
                                ));
                            }
                            for choice in chunk2 {
                                let value_id = self.choice_values_ids.get(choice).unwrap();
                                column2 = column2.push(Radio::new(
                                    value_id,
                                    choice,
                                    self.choice_values_ids
                                        .get(self.choice_values.get(&radio_id).unwrap()),
                                    move |value_id| Message::ChoicePressed(radio_id, *value_id),
                                ));
                            }

                            let row = Row::new().push(column1).push(column2);
                            content = content.push(row);

                        // for choice in choices.iter() {
                        //     let value_id = self.choice_values_ids.get(choice).unwrap();
                        //     content = content.push(Radio::new(
                        //         value_id,
                        //         choice,
                        //         self.choice_values_ids
                        //             .get(self.choice_values.get(&radio_id).unwrap()),
                        //         move |value_id| Message::ChoicePressed(radio_id, *value_id),
                        //     ));
                        // }
                        } else if part_is_search_by_str(form_item) {
                            let id = self
                                .search_by_str_ids
                                .get(form_item.name.as_ref().unwrap())
                                .unwrap()
                                .clone();
                            let input_value = self.search_by_str_values.get(&id).unwrap().clone();

                            content = content.push(TextInput::new(
                                id,
                                "Saisissez le nom ici: ",
                                &input_value,
                                Message::SearchByStrInputPressed,
                                if self.search_by_str_selected == id {
                                    blink_char
                                } else {
                                    None
                                },
                            ));

                            let mut choices: Vec<String> =
                                form_item.choices.as_ref().unwrap().clone();
                            let current_value = self.search_by_str_values.get(&id);
                            if current_value.is_some() {
                                choices = choices
                                    .into_iter()
                                    .filter(|string_| {
                                        string_
                                            .to_lowercase()
                                            .matches(current_value.unwrap().to_lowercase().as_str())
                                            .collect::<String>()
                                            .len()
                                            != 0
                                    })
                                    .collect();
                            }
                            for choice in choices.iter() {
                                let label = choice.to_string();
                                let choice_id =
                                    self.search_by_str_button_ids.get(&label).unwrap().clone();
                                content = content.push(
                                    StateLessButton::new(
                                        self.search_by_str_button_pressed == choice_id,
                                        &label,
                                        Message::SearchByStrButtonPressed(id, choice_id),
                                        Message::SearchByStrButtonReleased(id, choice_id),
                                    )
                                    .width(768)
                                    .class(state_less_button::Class::Positive),
                                );
                            }
                        }
                    }
                } else if part_is_link(item) {
                    let label = item
                        .label
                        .as_ref()
                        .unwrap_or(item.text.as_ref().unwrap_or(&"Continuer".to_string()))
                        .clone();
                    let display_label = if item.text.is_some() && item.label.is_some() {
                        format!(
                            "{}: {}",
                            item.label.as_ref().unwrap().clone(),
                            item.text.as_ref().unwrap().clone()
                        )
                    } else {
                        label.clone()
                    };
                    let id = *self.link_button_ids.get(&label).unwrap();
                    let mut display_normal_button = false;

                    if self.current_link_group_name.is_some() {
                        // display normal button if have same group name
                        if self.current_link_group_name.as_ref() == item.link_group_name.as_ref() {
                            display_normal_button = true;
                        }
                    } else {
                        if let Some(link_group_name) = item.link_group_name.as_ref() {
                            if !replaced_by_group_names.contains(link_group_name) {
                                let group_button_id = *self
                                    .link_group_name_ids
                                    .get(&link_group_name.clone())
                                    .unwrap();
                                content = content.push(
                                    StateLessButton::new(
                                        self.link_group_button_pressed == group_button_id,
                                        &link_group_name,
                                        Message::GroupLinkButtonPressed(group_button_id),
                                        Message::GroupLinkButtonReleased(link_group_name.clone()),
                                    )
                                    .width(768)
                                    .class(state_less_button::Class::Primary),
                                );
                                replaced_by_group_names.push(link_group_name.clone());
                            }
                        } else {
                            display_normal_button = true;
                        }
                    }

                    if display_normal_button {
                        content = content.push(
                            StateLessButton::new(
                                self.link_button_pressed == id,
                                &display_label,
                                Message::LinkButtonPressed(id),
                                Message::LinkButtonReleased(
                                    item.form_action.as_ref().unwrap().clone(),
                                ),
                            )
                            .width(768)
                            .class(state_less_button::Class::Primary),
                        );
                    }
                } else if part_is_go_back_to_zone(item) {
                    back_to_zone_button =
                        Some(
                            item.name
                                .as_ref()
                                .unwrap_or(item.text.as_ref().unwrap_or(
                                    &"Retourner sur l'écran de déplacements".to_string(),
                                ))
                                .clone(),
                        );
                }
            }

            if must_add_submit {
                content = content.push(
                    Button::new(&mut self.submit_button, &submit_label)
                        .on_press(Message::SubmitButtonPressed)
                        .width(768)
                        .class(button::Class::Primary),
                );
            }

            for link_item in description.footer_links.iter() {
                let label = link_item.label.as_ref().unwrap_or(&" ".to_string()).clone();
                if link_item.go_back_zone {
                    back_to_zone_button = Some(label.clone());
                    break;
                }
                let id = *self.link_button_ids.get(&label).unwrap();
                content = content.push(
                    StateLessButton::new(
                        self.link_button_pressed == id,
                        &label,
                        Message::LinkButtonPressed(id),
                        Message::LinkButtonReleased(
                            link_item.form_action.as_ref().unwrap().clone(),
                        ),
                    )
                    .width(768)
                    .class(state_less_button::Class::Primary),
                );
            }

            if self.current_link_group_name.is_some() {
                content = content.push(
                    Button::new(&mut self.back_from_group_by_button, "Retour")
                        .on_press(Message::GoBackFromGroupButtonPressed)
                        .width(768)
                        .class(button::Class::Secondary),
                );
            }

            if let Some(label) = back_to_zone_button {
                content = content.push(
                    Button::new(&mut self.go_back_zone_button, &label)
                        .on_press(Message::GoBackZoneButtonPressed)
                        .width(768)
                        .class(button::Class::Secondary),
                );
            }

            let submit_info = if must_add_submit {
                ", Entrer: Valider"
            } else {
                ""
            };
            let info = Column::new()
                .max_width(window.width() as u32)
                .height(20)
                .push(
                    Text::new(&format!(
                        "Echap: retour, ↑/↓/roulette: défilement{}",
                        submit_info
                    ))
                    .size(20)
                    .color(Color::WHITE)
                    .horizontal_alignment(HorizontalAlignment::Right)
                    .vertical_alignment(VerticalAlignment::Top),
                );

            Column::new()
                .width(window.width() as u32)
                .padding(0)
                .spacing(2)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .push(info)
                .push(content)
                .into()
        }
    }

    fn teardown(&mut self) {}
}
