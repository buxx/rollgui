use crate::ui::renderer::Renderer;
use crate::ui::widget::link;
use crate::ui::widget::text;
use coffee::graphics::{
    Color, HorizontalAlignment, Point, Rectangle, Sprite, Text, VerticalAlignment,
};
use coffee::ui::core::MouseCursor;

const GRAY1_X: u16 = 180;
const GRAY1_Y: u16 = 1000;
const GRAY2_X: u16 = 180;
const GRAY2_Y: u16 = 1001;
const GRAY3_X: u16 = 180;
const GRAY3_Y: u16 = 1002;

impl link::Renderer for Renderer {
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle<f32>,
        label: &str,
        class: Option<text::Class>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> MouseCursor {
        if let Some(class) = class {
            let (x, y) = match class {
                text::Class::BgGray1 => (GRAY1_X, GRAY1_Y),
                text::Class::BgGray2 => (GRAY2_X, GRAY2_Y),
                text::Class::BgGray3 => (GRAY3_X, GRAY3_Y),
                _ => panic!("not implemented"),
            };

            self.sprites.add(Sprite {
                source: Rectangle {
                    x,
                    y,
                    width: 1,
                    height: 1,
                },
                position: Point::new(bounds.x, bounds.y),
                scale: (bounds.width, bounds.height),
            });
        }

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
            horizontal_alignment,
            vertical_alignment,
            ..Text::default()
        });

        if mouse_over {
            MouseCursor::Pointer
        } else {
            MouseCursor::OutOfBounds
        }
    }
}
