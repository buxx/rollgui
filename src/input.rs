use coffee::graphics::Point;
use coffee::input;
use coffee::input::{keyboard, mouse, Event, Input};
use std::collections::HashSet;

#[derive(Debug)]
pub struct MyGameInput {
    pub cursor_position: Point,
    pub mouse_wheel: Point,
    pub keys_pressed: HashSet<keyboard::KeyCode>,
    last_key_code: Option<keyboard::KeyCode>,
    pub key_code: Option<keyboard::KeyCode>,
    pub mouse_buttons_pressed: HashSet<mouse::Button>,
    pub text_buffer: String,
}

impl Input for MyGameInput {
    fn new() -> Self {
        Self {
            cursor_position: Point::new(0.0, 0.0),
            mouse_wheel: Point::new(0.0, 0.0),
            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            text_buffer: String::new(),
            last_key_code: None,
            key_code: None,
        }
    }

    fn update(&mut self, event: Event) {
        self.key_code = None;
        match event {
            input::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { x, y } => {
                    self.cursor_position = Point::new(x, y);
                }
                mouse::Event::Input { state, button } => match state {
                    input::ButtonState::Pressed => {
                        self.mouse_buttons_pressed.insert(button);
                    }
                    input::ButtonState::Released => {
                        self.mouse_buttons_pressed.remove(&button);
                    }
                },
                mouse::Event::WheelScrolled { delta_x, delta_y } => {
                    self.mouse_wheel = Point::new(delta_x, delta_y);
                }
                _ => {}
            },
            input::Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::TextEntered { character } => {
                    self.text_buffer.push(character);
                }
                keyboard::Event::Input { key_code, state } => match state {
                    input::ButtonState::Pressed => {
                        self.keys_pressed.insert(key_code);
                        self.last_key_code = Some(key_code);
                    }
                    input::ButtonState::Released => {
                        self.keys_pressed.remove(&key_code);
                        if let Some(last_key_code) = self.last_key_code {
                            if last_key_code == key_code {
                                self.key_code = Some(last_key_code);
                            }
                        }
                    }
                },
            },
            _ => {}
        }
    }

    fn clear(&mut self) {}
}
