use crate::engine::Engine;
use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use crate::ui::Column;
use crate::ui::Element;
use coffee::graphics::{Color, Frame, HorizontalAlignment, VerticalAlignment, Window};
use coffee::ui::{button, Align, Button, Justify, Text};
use coffee::Timer;

pub struct StartupEngine {
    local_server_button: button::State,
    s2_bux_fr_server_button: button::State,
    exit_button: button::State,
}

impl StartupEngine {
    pub fn new() -> Self {
        Self {
            local_server_button: button::State::new(),
            s2_bux_fr_server_button: button::State::new(),
            exit_button: button::State::new(),
        }
    }
}

impl Engine for StartupEngine {
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
    }

    fn update(&mut self, _window: &Window) -> Option<MainMessage> {
        None
    }

    fn interact(&mut self, _input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        None
    }

    fn react(&mut self, event: Message, _window: &mut Window) -> Option<MainMessage> {
        match event {
            Message::LocalServerPressed => {
                return Some(MainMessage::StartupToZone {
                    server_ip: "127.0.0.1".to_string(),
                    server_port: 5000,
                })
            }
            Message::S2BuxFrServerPressed => {
                return Some(MainMessage::StartupToZone {
                    server_ip: "91.121.134.31".to_string(),
                    server_port: 7431,
                })
            }
            Message::ExitMenuButtonPressed => return Some(MainMessage::ExitRequested),
            _ => {}
        }

        None
    }
    fn layout(&mut self, window: &Window) -> Element<Message> {
        let StartupEngine {
            local_server_button,
            s2_bux_fr_server_button,
            exit_button,
        } = self;

        Column::new()
            .width(window.width() as u32)
            .height(window.height() as u32)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .spacing(20)
            .push(
                Text::new("Rolling")
                    .size(50)
                    .height(60)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            )
            .push(
                Button::new(local_server_button, "serveur local")
                    .on_press(Message::LocalServerPressed)
                    .class(button::Class::Primary)
                    .width(175),
            )
            .push(
                Button::new(s2_bux_fr_server_button, "s2.bux.fr")
                    .on_press(Message::S2BuxFrServerPressed)
                    .class(button::Class::Primary)
                    .width(175),
            )
            .push(
                Button::new(exit_button, "Quitter")
                    .on_press(Message::ExitMenuButtonPressed)
                    .class(button::Class::Secondary)
                    .width(175),
            )
            .into()
    }

    fn teardown(&mut self) {}
}
