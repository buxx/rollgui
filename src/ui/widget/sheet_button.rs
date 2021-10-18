use coffee::graphics::{Point, Rectangle};
use coffee::input::{mouse, ButtonState};
use coffee::ui::core::{Align, Event, Hasher, Layout, MouseCursor, Node, Style, Widget};

use crate::message;
use crate::sheet::TileSheet;
use crate::ui::{renderer, Element};
use std::hash::Hash;

pub const NODE_HEIGHT: u32 = 32;
pub struct SheetButton {
    pressed: bool,
    tile1: Rectangle<u16>,
    tile2: Rectangle<u16>,
    on_press: message::Message,
    on_release: message::Message,
    style: Style,
}

impl std::fmt::Debug for SheetButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SheetButton")
            .field("pressed", &self.pressed)
            .field("tile1", &self.tile1)
            .field("tile2", &self.tile2)
            .field("on_press", &self.on_press)
            .field("on_release", &self.on_release)
            .field("style", &self.style)
            .finish()
    }
}

impl SheetButton {
    pub fn new(
        pressed: bool,
        tile_sheet: &TileSheet,
        classes1: &Vec<String>,
        classes2: &Vec<String>,
        on_press: message::Message,
        on_release: message::Message,
    ) -> Self {
        let mut tile1_position = tile_sheet.appearance("UNKNOWN").unwrap()[0];
        let mut tile2_position = tile_sheet.appearance("UNKNOWN").unwrap()[0];

        for class in classes1.iter().rev() {
            if let Some(positions) = tile_sheet.appearance(class) {
                tile1_position = positions[0];
                break;
            }
        }

        for class in classes2.iter().rev() {
            if let Some(positions) = tile_sheet.appearance(class) {
                tile2_position = positions[0];
                break;
            }
        }

        let tile1 = tile_sheet.sources[&tile1_position];
        let tile2 = tile_sheet.sources[&tile2_position];

        SheetButton {
            pressed,
            tile1,
            tile2,
            on_press,
            on_release,
            style: Style::default().min_width(64),
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
}

impl Widget<message::Message, renderer::Renderer> for SheetButton {
    fn node(&self, _renderer: &renderer::Renderer) -> Node {
        Node::new(self.style.height(NODE_HEIGHT))
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
            self.pressed,
            self.tile1,
            self.tile2,
        )
    }

    fn hash(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
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
        pressed: bool,
        tile1: Rectangle<u16>,
        tile2: Rectangle<u16>,
    ) -> MouseCursor;
}

impl<'a> From<SheetButton> for Element<'a> {
    fn from(button: SheetButton) -> Element<'a> {
        Element::new(button)
    }
}
