//! Displays action progress to your users.

use coffee::graphics::{Point, Rectangle};
use coffee::ui::core::{Element, Hasher, Layout, MouseCursor, Node, Style, Widget};

use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorClass {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    SimpleThin,
}

/// A widget that displays a progress of an action.
///
/// It implements [`Widget`] when the associated [`core::Renderer`] implements
/// the [`button::Renderer`] trait.
///
/// [`Widget`]: ../../core/trait.Widget.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
/// [`progress_bar::Renderer`]: trait.Renderer.html
/// # Example
///
/// ```
/// use coffee::ui::ProgressBar;
///
/// let progress = 0.75;
///
/// ProgressBar::new(progress);
/// ```
#[derive(Debug)]
pub struct ProgressBar {
    progress: f32,
    class: Class,
    color_class: ColorClass,
    style: Style,
}

impl ProgressBar {
    /// Creates a new [`ProgressBar`] with given progress.
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn new(progress: f32, class: Class, color_class: ColorClass) -> Self {
        ProgressBar {
            progress,
            class,
            color_class,
            style: Style::default().fill_width(),
        }
    }

    /// Sets the width of the [`ProgressBar`] in pixels.
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn width(mut self, width: u32) -> Self {
        self.style = self.style.width(width);
        self
    }

    /// Makes the [`ProgressBar`] fill the horizontal space of its container.
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn fill_width(mut self) -> Self {
        self.style = self.style.fill_width();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for ProgressBar
where
    Renderer: self::Renderer,
{
    fn node(&self, _renderer: &Renderer) -> Node {
        let height = match self.class {
            Class::SimpleThin => 20,
        };
        Node::new(self.style.height(height))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        renderer.draw(layout.bounds(), self.class, self.color_class, self.progress);
        MouseCursor::OutOfBounds
    }

    fn hash(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

/// The renderer of a [`ProgressBar`].
///
/// Your [`core::Renderer`] will need to implement this trait before being
/// able to use a [`ProgressBar`] in your user interface.
///
/// [`ProgressBar`]: struct.ProgressBar.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
pub trait Renderer {
    /// Draws a [`ProgressBar`].
    ///
    /// It receives:
    ///   * the bounds of the [`ProgressBar`]
    ///   * the progress of the [`ProgressBar`]
    ///   
    /// [`ProgressBar`]: struct.ProgressBar.html
    fn draw(
        &mut self,
        bounds: Rectangle<f32>,
        class: Class,
        color_class: ColorClass,
        progress: f32,
    );
}

impl<'a, Message, Renderer> From<ProgressBar> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(progress_bar: ProgressBar) -> Element<'a, Message, Renderer> {
        Element::new(progress_bar)
    }
}
