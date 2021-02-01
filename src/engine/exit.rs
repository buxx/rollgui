use crate::engine::Engine;
use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use crate::ui::widget::button;
use crate::ui::widget::button::Button;
use crate::ui::widget::text::Text;
use crate::ui::Column;
use crate::ui::Element;
use coffee::graphics::{Color, Frame, Image, Window};
use coffee::input::keyboard;
use coffee::ui::{Align, Justify};
use coffee::Timer;

pub struct ExitEngine {
    confirm_button: button::State,
    cancel_button: button::State,
}

impl ExitEngine {
    pub fn new() -> Self {
        Self {
            confirm_button: button::State::new(),
            cancel_button: button::State::new(),
        }
    }
}

impl Engine for ExitEngine {
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
    }

    fn update(&mut self, _window: &Window) -> Option<MainMessage> {
        None
    }

    fn interact(&mut self, input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        match input.key_code {
            Some(keyboard::KeyCode::Escape) => {
                input.key_code = None;
                return Some(MainMessage::DescriptionToZone {
                    request_clicks: None,
                });
            }
            _ => {}
        }

        None
    }

    fn react(&mut self, event: Message, _window: &mut Window) -> Option<MainMessage> {
        match event {
            Message::ConfirmButtonPressed => return Some(MainMessage::ToStartup),
            Message::CancelButtonPressed => {
                return Some(MainMessage::DescriptionToZone {
                    request_clicks: None,
                })
            }
            _ => {}
        }

        None
    }
    fn layout(&mut self, window: &Window, _illustration: Option<Image>) -> Element {
        let ExitEngine {
            confirm_button,
            cancel_button,
        } = self;

        Column::new()
            .max_width(768)
            .height(window.height() as u32)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .spacing(20)
            .push(
                Text::new("Voulez-vous vraiment quitter ?")
                    .size(50)
                    .height(60),
            )
            .push(
                Button::new(confirm_button, "Quitter")
                    .on_press(Message::ConfirmButtonPressed)
                    .class(button::Class::Primary),
            )
            .push(
                Button::new(cancel_button, "Retour")
                    .on_press(Message::CancelButtonPressed)
                    .class(button::Class::Secondary),
            )
            .into()
    }

    fn teardown(&mut self) {}
}
