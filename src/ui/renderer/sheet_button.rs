use crate::ui::renderer::Renderer;
use crate::ui::widget::sheet_button;
use coffee::graphics::{Point, Rectangle, Sprite};
use coffee::ui::core::MouseCursor;

const LEFT: Rectangle<u16> = Rectangle {
    x: 0,
    y: 1000,
    width: 9,
    height: 34,
};

const BACKGROUND: Rectangle<u16> = Rectangle {
    x: 9,
    y: 1000,
    width: 1,
    height: 34,
};

const RIGHT: Rectangle<u16> = Rectangle {
    x: 18,
    y: 1000,
    width: 9,
    height: 34,
};

const HEIGHT_SCALE: f32 = 1.0;

impl sheet_button::Renderer for Renderer {
    fn draw(
        &mut self,
        cursor_position: Point,
        mut bounds: Rectangle<f32>,
        pressed: bool,
        tile1: Rectangle<u16>,
        tile2: Rectangle<u16>,
    ) -> MouseCursor {
        let mouse_over = bounds.contains(cursor_position);
        if mouse_over {
            if pressed {
                bounds.y += 4.0;
            } else {
                bounds.y -= 1.0;
            }
        }

        self.sprites.add(Sprite {
            source: Rectangle {
                x: LEFT.x,
                y: LEFT.y,
                ..LEFT
            },
            position: Point::new(bounds.x, bounds.y),
            scale: (1.0, HEIGHT_SCALE),
        });

        self.sprites.add(Sprite {
            source: Rectangle {
                x: BACKGROUND.x,
                y: BACKGROUND.y,
                ..BACKGROUND
            },
            position: Point::new(bounds.x + LEFT.width as f32, bounds.y),
            scale: (50.0, HEIGHT_SCALE),
        });

        self.sprites.add(Sprite {
            source: Rectangle {
                x: RIGHT.x,
                y: RIGHT.y,
                ..RIGHT
            },
            position: Point::new(bounds.x + bounds.width - RIGHT.width as f32, bounds.y),
            scale: (1.0, HEIGHT_SCALE),
        });

        self.sprites.add(Sprite {
            source: tile1,
            position: Point::new(bounds.x as f32 + 0.0, bounds.y as f32 + 0.0),
            scale: (1.0, 1.0),
        });

        self.sprites.add(Sprite {
            source: tile2,
            position: Point::new(bounds.x as f32 + 31.0, bounds.y as f32 + 0.0),
            scale: (1.0, 1.0),
        });

        if mouse_over {
            MouseCursor::Pointer
        } else {
            MouseCursor::OutOfBounds
        }
    }
}
