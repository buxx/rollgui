use coffee::graphics::{HorizontalAlignment, Point, Rectangle, VerticalAlignment};
use coffee::input::{mouse, ButtonState};
use coffee::ui::core::{Align, Event, Hasher, Layout, MouseCursor, Node, Style, Widget};

use crate::message;
use crate::ui::widget::text;
use crate::ui::{renderer, Element};
use std::hash::Hash;

pub struct Link {
    pressed: bool,
    label: String,
    on_press: message::Message,
    on_release: message::Message,
    style: Style,
    class: Option<text::Class>,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl std::fmt::Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("label", &self.label)
            .field("on_press", &self.on_press)
            .field("style", &self.style)
            .field("class", &self.class)
            .finish()
    }
}

impl Link {
    pub fn new(
        pressed: bool,
        label: &str,
        on_press: message::Message,
        on_release: message::Message,
        class: Option<text::Class>,
    ) -> Self {
        Link {
            pressed,
            label: String::from(label),
            on_press,
            on_release,
            style: Style::default().min_width(100),
            class,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.style = self.style.width(width);
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.style = self.style.fill_width();
        self
    }

    pub fn align_self(mut self, align: Align) -> Self {
        self.style = self.style.align_self(align);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.style = self.style.height(height);
        self
    }

    pub fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl Widget<message::Message, renderer::Renderer> for Link {
    fn node(&self, _renderer: &renderer::Renderer) -> Node {
        Node::new(self.style)
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
                state,
            }) => {
                let bounds = layout.bounds();
                let on_it = bounds.contains(cursor_position);

                match state {
                    ButtonState::Pressed => {
                        if on_it {
                            messages.push(self.on_press.clone());
                        }
                    }
                    ButtonState::Released => {
                        if on_it && self.pressed {
                            messages.push(self.on_release.clone());
                        }
                    }
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
        renderer.draw(
            cursor_position,
            layout.bounds(),
            &self.label,
            self.class,
            self.horizontal_alignment,
            self.vertical_alignment,
        )
    }

    fn hash(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

pub trait Renderer {
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle<f32>,
        label: &str,
        class: Option<text::Class>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> MouseCursor;
}

impl<'a> From<Link> for Element<'a> {
    fn from(link: Link) -> Element<'a> {
        Element::new(link)
    }
}
