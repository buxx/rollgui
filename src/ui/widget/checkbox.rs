//! Show toggle controls using checkboxes.
use std::hash::Hash;

use coffee::graphics::{
    Color, HorizontalAlignment, Point, Rectangle, VerticalAlignment,
};
use coffee::input::{mouse, ButtonState};
use coffee::ui::core::{
    Align, Event, Hasher, Layout, MouseCursor, Node, Widget,
};
use crate::message;
use crate::ui::widget::text;
use crate::ui::{renderer, Row, Column, Element};
use crate::ui::widget::text::Text;

/// A box that can be checked.
///
/// It implements [`Widget`] when the [`core::Renderer`] implements the
/// [`checkbox::Renderer`] trait.
///
/// [`Widget`]: ../../core/trait.Widget.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
/// [`checkbox::Renderer`]: trait.Renderer.html
///
/// # Example
///
/// ```
/// use coffee::graphics::Color;
/// use coffee::ui::Checkbox;
///
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// let is_checked = true;
///
/// Checkbox::new(is_checked, "Toggle me!", Message::CheckboxToggled)
///     .label_color(Color::BLACK);
/// ```
///
/// ![Checkbox drawn by the built-in renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/checkbox.png?raw=true)
pub struct Checkbox {
    is_checked: bool,
    on_toggle: Box<dyn Fn(bool) -> message::Message>,
    label: String,
    label_color: Color,
}

impl std::fmt::Debug for Checkbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("label", &self.label)
            .field("label_color", &self.label_color)
            .finish()
    }
}

impl Checkbox {
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled.
    ///   It receives the new state of the [`Checkbox`] and must produce a
    ///   `Message`.
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn new<F>(is_checked: bool, label: &str, f: F) -> Self
    where
        F: 'static + Fn(bool) -> message::Message,
    {
        Checkbox {
            is_checked,
            on_toggle: Box::new(f),
            label: String::from(label),
            label_color: Color::WHITE,
        }
    }

    /// Sets the [`Color`] of the label of the [`Checkbox`].
    ///
    /// [`Color`]: ../../../../graphics/struct.Color.html
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = color;
        self
    }
}

impl Widget<message::Message, renderer::Renderer> for Checkbox {
    fn node(&self, renderer: &renderer::Renderer) -> Node {
        Row::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(Column::new().width(28).height(28))
            .push(Text::new(&self.label))
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
                let mouse_over = layout
                    .children()
                    .any(|child| child.bounds().contains(cursor_position));

                if mouse_over {
                    messages.push((self.on_toggle)(!self.is_checked));
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
        let children: Vec<_> = layout.children().collect();

        let text_bounds = children[1].bounds();

        text::Renderer::draw(
            renderer,
            text_bounds,
            &self.label,
            20.0,
            self.label_color,
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
        );

        self::Renderer::draw(
            renderer,
            cursor_position,
            children[0].bounds(),
            text_bounds,
            self.is_checked,
        )
    }

    fn hash(&self, state: &mut Hasher) {
        self.label.hash(state);
    }
}

/// The renderer of a [`Checkbox`].
///
/// Your [`core::Renderer`] will need to implement this trait before being
/// able to use a [`Checkbox`] in your user interface.
///
/// [`Checkbox`]: struct.Checkbox.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
pub trait Renderer {
    /// Draws a [`Checkbox`].
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Checkbox`]
    ///   * the bounds of the label of the [`Checkbox`]
    ///   * whether the [`Checkbox`] is checked or not
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle<f32>,
        label_bounds: Rectangle<f32>,
        is_checked: bool,
    ) -> MouseCursor;
}

impl<'a> From<Checkbox> for Element<'a> {
    fn from(checkbox: Checkbox) -> Element<'a> {
        Element::new(checkbox)
    }
}
