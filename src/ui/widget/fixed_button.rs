//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`] and a [`Class`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html
//! [`Class`]: enum.Class.html

use coffee::graphics::{Point, Rectangle};
use coffee::input::{mouse, ButtonState};
use coffee::ui::core::{Align, Event, Hasher, Layout, MouseCursor, Node, Style, Widget};

use crate::message;
use crate::ui::{renderer, Element};
use std::hash::Hash;

/// A generic widget that produces a message when clicked.
///
/// It implements [`Widget`] when the associated [`core::Renderer`] implements
/// the [`button::Renderer`] trait.
///
/// [`Widget`]: ../../core/trait.Widget.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
/// [`button::Renderer`]: trait.Renderer.html
///
/// # Example
///
/// ```
/// use coffee::ui::{button, Button};
///
/// pub enum Message {
///     ButtonClicked,
/// }
///
/// let state = &mut button::State::new();
///
/// Button::new(state, "Click me!")
///     .on_press(Message::ButtonClicked);
/// ```
///
/// ![Button drawn by the built-in renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/button.png?raw=true)
pub struct Button<'a> {
    state: &'a mut State,
    label: String,
    class: Class,
    on_press: Option<message::Message>,
    style: Style,
}

impl<'a> std::fmt::Debug for Button<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("state", &self.state)
            .field("label", &self.label)
            .field("class", &self.class)
            .field("on_press", &self.on_press)
            .field("style", &self.style)
            .finish()
    }
}

impl<'a> Button<'a> {
    /// Creates a new [`Button`] with some local [`State`] and the given label.
    ///
    /// The default [`Class`] of a new [`Button`] is [`Class::Primary`].
    ///
    /// [`Button`]: struct.Button.html
    /// [`State`]: struct.State.html
    /// [`Class`]: enum.Class.html
    /// [`Class::Primary`]: enum.Class.html#variant.Primary
    pub fn new(state: &'a mut State, label: &str) -> Self {
        Button {
            state,
            label: String::from(label),
            class: Class::Back,
            on_press: None,
            style: Style::default().min_width(100),
        }
    }

    /// Sets the width of the [`Button`] in pixels.
    ///
    /// [`Button`]: struct.Button.html
    pub fn width(mut self, width: u32) -> Self {
        self.style = self.style.width(width);
        self
    }

    /// Makes the [`Button`] fill the horizontal space of its container.
    ///
    /// [`Button`]: struct.Button.html
    pub fn fill_width(mut self) -> Self {
        self.style = self.style.fill_width();
        self
    }

    /// Sets the alignment of the [`Button`] itself.
    ///
    /// This is useful if you want to override the default alignment given by
    /// the parent container.
    ///
    /// [`Button`]: struct.Button.html
    pub fn align_self(mut self, align: Align) -> Self {
        self.style = self.style.align_self(align);
        self
    }

    /// Sets the [`Class`] of the [`Button`].
    ///
    ///
    /// [`Button`]: struct.Button.html
    /// [`Class`]: enum.Class.html
    pub fn class(mut self, class: Class) -> Self {
        self.class = class;
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// [`Button`]: struct.Button.html
    pub fn on_press(mut self, msg: message::Message) -> Self {
        self.on_press = Some(msg);
        self
    }
}

impl<'a> Widget<message::Message, renderer::Renderer> for Button<'a> {
    fn node(&self, _renderer: &renderer::Renderer) -> Node {
        Node::new(self.style.height(50))
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
                if let Some(on_press) = self.on_press.clone() {
                    let bounds = layout.bounds();

                    match state {
                        ButtonState::Pressed => {
                            self.state.is_pressed = bounds.contains(cursor_position);
                        }
                        ButtonState::Released => {
                            let is_clicked =
                                self.state.is_pressed && bounds.contains(cursor_position);

                            self.state.is_pressed = false;

                            if is_clicked {
                                messages.push(on_press);
                            }
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
            self.state,
            &self.label,
            self.class,
        )
    }

    fn hash(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

/// The local state of a [`Button`].
///
/// [`Button`]: struct.Button.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }

    /// Returns whether the associated [`Button`] is currently being pressed or
    /// not.
    ///
    /// [`Button`]: struct.Button.html
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
}

/// The type of a [`Button`].
///
/// ![Different buttons drawn by the built-in renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/button_classes.png?raw=true)
///
/// [`Button`]: struct.Button.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Back,
    Zone,
    Item,
    Build,
    Character,
    Action,
    Affinity,
}

/// The renderer of a [`Button`].
///
/// Your [`core::Renderer`] will need to implement this trait before being
/// able to use a [`Button`] in your user interface.
///
/// [`Button`]: struct.Button.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
pub trait Renderer {
    /// Draws a [`Button`].
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Button`]
    ///   * the local state of the [`Button`]
    ///   * the label of the [`Button`]
    ///   * the [`Class`] of the [`Button`]
    ///
    /// [`Button`]: struct.Button.html
    /// [`State`]: struct.State.html
    /// [`Class`]: enum.Class.html
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle<f32>,
        state: &State,
        label: &str,
        class: Class,
    ) -> MouseCursor;
}

impl<'a> From<Button<'a>> for Element<'a> {
    fn from(button: Button<'a>) -> Element<'a> {
        Element::new(button)
    }
}
