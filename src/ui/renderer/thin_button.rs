use crate::ui::renderer::Renderer;
use crate::ui::widget::thin_button;
use coffee::graphics::{
    Color, HorizontalAlignment, Point, Rectangle, Sprite, Text, VerticalAlignment,
};
use coffee::ui::core::MouseCursor;

const LEFT: Rectangle<u16> = Rectangle {
    x: 0,
    y: 1199,
    width: 2,
    height: 23,
};

const BACKGROUND: Rectangle<u16> = Rectangle {
    x: LEFT.width,
    y: LEFT.y,
    width: 1,
    height: LEFT.height,
};

const RIGHT: Rectangle<u16> = Rectangle {
    x: 47,
    y: 1199,
    width: 2,
    height: 23,
};

impl thin_button::Renderer for Renderer {
    fn draw(
        &mut self,
        cursor_position: Point,
        mut bounds: Rectangle<f32>,
        state: &thin_button::State,
        label: &str,
        class: thin_button::Class,
    ) -> MouseCursor {
        let mouse_over = bounds.contains(cursor_position);

        let mut state_offset = 0;

        if mouse_over {
            if state.is_pressed() {
                bounds.y += 4.0;
                state_offset = RIGHT.x + RIGHT.width;
            } else {
                bounds.y -= 1.0;
            }
        }

        let class_index = match class {
            thin_button::Class::Primary => 0,
            thin_button::Class::Secondary => 1,
            thin_button::Class::Positive => 2,
        };

        self.sprites.add(Sprite {
            source: Rectangle {
                x: LEFT.x + state_offset,
                y: LEFT.y + class_index * LEFT.height,
                ..LEFT
            },
            position: Point::new(bounds.x, bounds.y),
            scale: (1.0, 1.0),
        });

        self.sprites.add(Sprite {
            source: Rectangle {
                x: BACKGROUND.x + state_offset,
                y: BACKGROUND.y + class_index * BACKGROUND.height,
                ..BACKGROUND
            },
            position: Point::new(bounds.x + LEFT.width as f32, bounds.y),
            scale: (bounds.width - (LEFT.width + RIGHT.width) as f32, 1.0),
        });

        self.sprites.add(Sprite {
            source: Rectangle {
                x: RIGHT.x + state_offset,
                y: RIGHT.y + class_index * RIGHT.height,
                ..RIGHT
            },
            position: Point::new(bounds.x + bounds.width - RIGHT.width as f32, bounds.y),
            scale: (1.0, 1.0),
        });

        self.font.borrow_mut().add(Text {
            content: label,
            position: Point::new(bounds.x, bounds.y - 1.0),
            bounds: (bounds.width, bounds.height),
            color: if mouse_over {
                Color::WHITE
            } else {
                Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                }
            },
            size: 17.0,
            horizontal_alignment: HorizontalAlignment::Center,
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
