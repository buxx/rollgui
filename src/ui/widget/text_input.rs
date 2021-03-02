//! Create text input.
use crate::message;
use crate::ui::renderer;
use crate::ui::Element;
use crate::ui::Row;
use coffee::graphics::{Color, HorizontalAlignment, Point, Rectangle, VerticalAlignment};
use coffee::input::{mouse, ButtonState};
use coffee::ui::core::{Align, Event, Hasher, Layout, MouseCursor, Node, Widget};

use crate::ui::widget::text;
use crate::ui::widget::text::Text;
use std::hash::Hash;

/// A text input.
pub struct TextInput<Id> {
    id: Id,
    on_selected: Box<dyn Fn(Id) -> message::Message>,
    label: String,
    value: String,
    color: Color,
    hover_color: Color,
    blink_char: Option<char>,
    class: Option<text::Class>,
    height: u32,
    is_password: bool,
}

impl<I> std::fmt::Debug for TextInput<I>
where
    I: Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("label", &self.label)
            .field("value", &self.value)
            .field("color", &self.color)
            .field("hover_color", &self.hover_color)
            .field("is_password", &self.is_password)
            .finish()
    }
}

impl<I> TextInput<I> {
    pub fn new<S>(
        id: I,
        label: &str,
        value: &str,
        on_selected: S,
        blink_char: Option<char>,
        class: Option<text::Class>,
    ) -> Self
    where
        S: 'static + Fn(I) -> message::Message,
    {
        TextInput {
            id,
            on_selected: Box::new(on_selected),
            label: String::from(label),
            value: String::from(value),
            color: Color::WHITE,
            hover_color: Color::GREEN,
            blink_char,
            class,
            height: 25,
            is_password: false,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn is_password(mut self, is_password: bool) -> Self {
        self.is_password = is_password;
        self
    }
}

impl<I> Widget<message::Message, renderer::Renderer> for TextInput<I>
where
    I: Copy,
{
    fn node(&self, renderer: &renderer::Renderer) -> Node {
        Row::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(Text::new(format!("{}: {}", self.label, self.value).as_str()).height(self.height))
            .node(renderer)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<message::Message>,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state: ButtonState::Pressed,
            }) => {
                if layout.bounds().contains(cursor_position) {
                    messages.push((self.on_selected)(self.id))
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut renderer::Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> MouseCursor {
        let hover = layout.bounds().contains(cursor_position);
        let color = if hover { self.hover_color } else { self.color };
        let value = if !self.is_password {
            self.value.clone()
        } else {
            "*".repeat(self.value.len())
        };
        let mut text = format!("{}: {}", self.label, value);
        if let Some(blink_char) = self.blink_char {
            text.push(blink_char);
        }

        text::Renderer::draw(
            renderer,
            layout.bounds(),
            text.as_str(),
            20.0,
            color,
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
            self.class,
        );

        if hover {
            MouseCursor::Pointer
        } else {
            MouseCursor::OutOfBounds
        }
    }

    fn hash(&self, state: &mut Hasher) {
        self.label.hash(state);
    }
}

pub trait Renderer {
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle<f32>,
        label_bounds: Rectangle<f32>,
        is_selected: bool,
    ) -> MouseCursor;
}

impl<'a, I: 'a> From<TextInput<I>> for Element<'a>
where
    I: Copy,
{
    fn from(text_input: TextInput<I>) -> Element<'a> {
        Element::new(text_input)
    }
}
