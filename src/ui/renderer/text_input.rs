use crate::ui::renderer::Renderer;
use coffee::graphics::{Point, Rectangle};
use coffee::ui::core::MouseCursor;
use crate::ui::widget::text_input;

impl text_input::Renderer for Renderer {
    fn draw(
        &mut self,
        cursor_position: Point,
        _bounds: Rectangle<f32>,
        bounds_with_label: Rectangle<f32>,
        _is_selected: bool,
    ) -> MouseCursor {
        // FIXME BS NOW: this code is used ?!
        let mouse_over = bounds_with_label.contains(cursor_position);

        if mouse_over {
            MouseCursor::Pointer
        } else {
            MouseCursor::OutOfBounds
        }
    }
}
