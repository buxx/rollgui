use crate::engine::Engine;
use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use crate::server;
use crate::ui::widget::button;
use crate::ui::widget::button::Button;
use crate::ui::widget::text::Text;
use crate::ui::widget::text_input::TextInput;
use crate::ui::{Column, Element, Row};
use coffee::graphics::{Batch, Color, Frame, Image, Point, Rectangle, Sprite, Window};
use coffee::input::keyboard;
use coffee::ui::{Align, Justify};
use coffee::Timer;
use std::time::Instant;

const BLINK_MS: u128 = 250;

pub struct LoginEngine {
    address: server::ServerAddress,
    client: server::client::Client,
    login_button: button::State,
    password_lost_button: button::State,
    create_account_button: button::State,
    cancel_button: button::State,
    login_input_text: String,
    login_input_text_is_selected: bool,
    password_input_text: String,
    password_input_text_is_selected: bool,
    blink_time: Instant,
    error_message: Option<String>,
    message: Option<String>,
}

impl LoginEngine {
    pub fn new(
        address: server::ServerAddress,
        message: Option<String>,
        default_login: String,
    ) -> Self {
        Self {
            address: address.clone(),
            client: server::client::Client::new(address.clone(), ("".to_string(), "".to_string())),
            login_button: button::State::new(),
            password_lost_button: button::State::new(),
            create_account_button: button::State::new(),
            cancel_button: button::State::new(),
            login_input_text: default_login.clone(),
            login_input_text_is_selected: false,
            password_input_text: "".to_string(),
            password_input_text_is_selected: default_login != "",
            blink_time: Instant::now(),
            error_message: None,
            message,
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

    fn submit(&mut self) -> Option<MainMessage> {
        self.error_message = None;
        let credentials = (
            self.login_input_text.trim().to_string(),
            self.password_input_text.trim().to_string(),
        );
        match self.client.get_current_character_id((
            self.login_input_text.clone(),
            self.password_input_text.clone(),
        )) {
            Ok(current_character_id) => {
                let current_character_id_ = if current_character_id == "" {
                    None
                } else {
                    Some(current_character_id)
                };
                return Some(MainMessage::EnterServer {
                    credentials,
                    character_id: current_character_id_,
                });
            }
            Err(server::client::ClientError::Unauthorized) => {
                self.error_message = Some("Mauvais login ou mot de passe ?".to_string());
            }
            Err(client_error) => {
                self.error_message = Some(server::client::ClientError::get_message(&client_error));
            }
        }

        None
    }
}

impl Engine for LoginEngine {
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer, illustration: Option<Image>) {
        frame.clear(Color::BLACK);

        if let Some(illustration) = illustration {
            let illustration_width = illustration.width();
            let illustration_height = illustration.height();
            let mut batch = Batch::new(illustration);
            batch.extend(vec![Sprite {
                source: Rectangle {
                    x: 0,
                    y: 0,
                    width: illustration_width,
                    height: illustration_height,
                },
                position: Point::new(0.0, 0.0),
                scale: (
                    frame.width() / illustration_width as f32,
                    frame.height() / illustration_height as f32,
                ),
            }]);
            batch.draw(&mut frame.as_target());
        };
    }

    fn update(&mut self, _window: &Window) -> Option<MainMessage> {
        None
    }

    fn interact(&mut self, input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        if !input.text_buffer.is_empty() {
            let input_text = if self.login_input_text_is_selected {
                &mut self.login_input_text
            } else {
                &mut self.password_input_text
            };
            for c in input.text_buffer.chars() {
                match c {
                    // Match ASCII backspace and delete from the text buffer
                    '\u{8}' => {
                        input_text.pop();
                    }
                    // Tabulation | Enter | Escape
                    '\t' | '\r' | '\u{1b}' => {}
                    _ => {
                        input_text.push(c);
                    }
                }
            }

            input.text_buffer = String::new();
        }

        match input.key_code {
            Some(keyboard::KeyCode::Escape) => {
                input.key_code = None;
                return Some(MainMessage::ToStartup);
            }
            Some(keyboard::KeyCode::Tab) => {
                input.key_code = None;
                self.login_input_text_is_selected = !self.login_input_text_is_selected;
                self.password_input_text_is_selected = !self.password_input_text_is_selected;
            }
            Some(keyboard::KeyCode::Return) => {
                input.key_code = None;
                return self.submit();
            }
            _ => {}
        }

        None
    }

    fn react(&mut self, event: Message, _window: &mut Window) -> Option<MainMessage> {
        match event {
            Message::LoginInputSelected { .. } => {
                self.login_input_text_is_selected = true;
                self.password_input_text_is_selected = false;
                self.error_message = None;
            }
            Message::PasswordInputSelected { .. } => {
                self.login_input_text_is_selected = false;
                self.password_input_text_is_selected = true;
                self.error_message = None;
            }
            Message::ConfirmButtonPressed => {
                return self.submit();
            }
            Message::PasswordLostButtonPressed => {
                let description = self
                    .client
                    .describe(
                        &format!("/account/password_lost?login={}", self.login_input_text),
                        None,
                        None,
                    )
                    .unwrap();
                return Some(MainMessage::ToDescriptionWithDescription {
                    description,
                    back_url: None,
                    client: self.client.clone(),
                });
            }
            Message::CreateAccountButtonPressed => {
                return Some(MainMessage::CreateAccount {
                    address: self.address.clone(),
                })
            }
            Message::ExitMenuButtonPressed => return Some(MainMessage::ExitRequested),
            _ => {}
        }

        None
    }

    fn layout(&mut self, window: &Window, _illustration: Option<Image>) -> Element {
        let blink_char = self.get_blink_char();
        let mut column = Column::new()
            .width(window.width() as u32)
            .height(window.height() as u32)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .spacing(20)
            .push(Text::new("").size(30).height(30).width(600));

        if let Some(error_message) = self.error_message.as_ref() {
            column = column.push(Text::new(error_message).color(Color::RED))
        } else if let Some(message) = self.message.as_ref() {
            column = column.push(Text::new(message).color(Color::GREEN))
        }

        column = column
            .push(
                Row::new()
                    .push(TextInput::new(
                        0,
                        "Login/Email",
                        &self.login_input_text,
                        Message::LoginInputSelected,
                        if self.login_input_text_is_selected {
                            blink_char
                        } else {
                            None
                        },
                        None,
                    ))
                    .width(500),
            )
            .push(
                Row::new()
                    .push(
                        TextInput::new(
                            1,
                            "Mot de passe",
                            &self.password_input_text,
                            Message::PasswordInputSelected,
                            if self.password_input_text_is_selected {
                                blink_char
                            } else {
                                None
                            },
                            None,
                        )
                        .is_password(true),
                    )
                    .width(500),
            )
            .push(
                Button::new(&mut self.login_button, "S'identifier")
                    .on_press(Message::ConfirmButtonPressed)
                    .class(button::Class::Primary)
                    .width(300),
            )
            .push(
                Button::new(&mut self.password_lost_button, "Mot de passe perdu")
                    .on_press(Message::PasswordLostButtonPressed)
                    .class(button::Class::Primary)
                    .width(300),
            )
            .push(
                Button::new(&mut self.create_account_button, "Créer un compte")
                    .on_press(Message::CreateAccountButtonPressed)
                    .class(button::Class::Primary)
                    .width(300),
            )
            .push(
                Button::new(&mut self.cancel_button, "Quitter")
                    .on_press(Message::ExitMenuButtonPressed)
                    .class(button::Class::Secondary)
                    .width(200),
            );

        column.into()
    }

    fn teardown(&mut self) {
        //
    }
}
