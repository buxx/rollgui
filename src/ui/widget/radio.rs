//! Create choices using radio buttons.
use coffee::graphics::{Color, HorizontalAlignment, Point, Rectangle, VerticalAlignment};
use coffee::input::{mouse, ButtonState};
use coffee::ui::core::{Align, Event, Hasher, Layout, MouseCursor, Node, Widget};

use crate::message;
use crate::ui::widget::text;
use crate::ui::widget::text::Text;
use crate::ui::{renderer, Column, Element, Row};
use std::hash::Hash;

/// A circular button representing a choice.
///
/// It implements [`Widget`] when the [`core::Renderer`] implements the
/// [`radio::Renderer`] trait.
///
/// [`Widget`]: ../../core/trait.Widget.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
/// [`radio::Renderer`]: trait.Renderer.html
///
/// # Example
/// ```
/// use coffee::graphics::Color;
/// use coffee::ui::{Column, Radio};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum Choice {
///     A,
///     B,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// pub enum Message {
///     RadioSelected(Choice),
/// }
///
/// let selected_choice = Some(Choice::A);
///
/// Column::new()
///     .spacing(20)
///     .push(
///         Radio::new(Choice::A, "This is A", selected_choice, Message::RadioSelected)
///             .label_color(Color::BLACK),
///     )
///     .push(
///         Radio::new(Choice::B, "This is B", selected_choice, Message::RadioSelected)
///             .label_color(Color::BLACK),
///     );
/// ```
///
/// ![Checkbox drawn by the built-in renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/radio.png?raw=true)
pub struct Radio {
    is_selected: bool,
    on_click: message::Message,
    label: String,
    label_color: Color,
}

impl std::fmt::Debug for Radio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Radio")
            .field("is_selected", &self.is_selected)
            .field("on_click", &self.on_click)
            .field("label", &self.label)
            .field("label_color", &self.label_color)
            .finish()
    }
}

impl Radio {
    /// Creates a new [`Radio`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Radio`] button
    ///   * the label of the [`Radio`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Radio`] is selected. It
    ///   receives the value of the radio and must produce a `Message`.
    ///
    /// [`Radio`]: struct.Radio.html
    pub fn new<F, V>(value: V, label: &str, selected: Option<V>, f: F) -> Self
    where
        V: Eq + Copy,
        F: 'static + Fn(V) -> message::Message,
    {
        Radio {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: String::from(label),
            label_color: Color::WHITE,
        }
    }

    /// Sets the [`Color`] of the label of the [`Radio`].
    ///
    /// [`Color`]: ../../../../graphics/struct.Color.html
    /// [`Radio`]: struct.Radio.html
    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = color;
        self
    }
}

impl Widget<message::Message, renderer::Renderer> for Radio {
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
                if layout.bounds().contains(cursor_position) {
                    messages.push(self.on_click.clone());
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

        let mut text_bounds = children[1].bounds();
        text_bounds.y -= 2.0;

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
            layout.bounds(),
            self.is_selected,
        )
    }

    fn hash(&self, state: &mut Hasher) {
        self.label.hash(state);
    }
}

/// The renderer of a [`Radio`] button.
///
/// Your [`core::Renderer`] will need to implement this trait before being
/// able to use a [`Radio`] button in your user interface.
///
/// [`Radio`]: struct.Radio.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
pub trait Renderer {
    /// Draws a [`Radio`] button.
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Radio`]
    ///   * the bounds of the label of the [`Radio`]
    ///   * whether the [`Radio`] is selected or not
    ///
    /// [`Radio`]: struct.Radio.html
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle<f32>,
        label_bounds: Rectangle<f32>,
        is_selected: bool,
    ) -> MouseCursor;
}

impl<'a> From<Radio> for Element<'a> {
    fn from(checkbox: Radio) -> Element<'a> {
        Element::new(checkbox)
    }
}
