use crate::ui::renderer::Renderer;
use crate::ui::widget::link;
use coffee::graphics::{Color, HorizontalAlignment, Point, Rectangle, Text, VerticalAlignment};
use coffee::ui::core::MouseCursor;

impl link::Renderer for Renderer {
    fn draw(&mut self, cursor_position: Point, bounds: Rectangle<f32>, label: &str) -> MouseCursor {
        let mouse_over = bounds.contains(cursor_position);
        let color = if mouse_over {
            Color::BLUE
        } else {
            Color::WHITE
        };

        self.font.borrow_mut().add(Text {
            content: label,
            position: Point::new(bounds.x, bounds.y),
            bounds: (bounds.width, bounds.height),
            color,
            size: 20.0,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
            ..Text::default()
        });

        if mouse_over {
            MouseCursor::Pointer
        } else {
            MouseCursor::OutOfBounds
        }
    }
}
