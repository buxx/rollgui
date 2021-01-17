use crate::ui::renderer::Renderer;
use crate::ui::widget::progress_bar;
use crate::ui::widget::progress_bar::{Class, ColorClass};
use coffee::graphics::{Point, Rectangle, Sprite};

const SIMPLE_THIN_LEFT_X: i32 = 196;
const SIMPLE_THIN_LEFT_WIDTH: i32 = 2;
const SIMPLE_THIN_Y: i32 = 0;
const SIMPLE_THIN_HEIGHT: i32 = 6;

const SIMPLE_THIN_CENTER_X: i32 = 198;
const SIMPLE_THIN_CENTER_WIDTH: i32 = 2;

const SIMPLE_THIN_RIGHT_X: i32 = 200;
const SIMPLE_THIN_RIGHT_WIDTH: i32 = 2;

impl progress_bar::Renderer for Renderer {
    fn draw(
        &mut self,
        bounds: Rectangle<f32>,
        class: progress_bar::Class,
        color_class: progress_bar::ColorClass,
        progress: f32,
    ) {
        let (left_x, left_width) = match class {
            Class::SimpleThin => (SIMPLE_THIN_LEFT_X, SIMPLE_THIN_LEFT_WIDTH),
        };
        let (center_x, center_width) = match class {
            Class::SimpleThin => (SIMPLE_THIN_CENTER_X, SIMPLE_THIN_CENTER_WIDTH),
        };
        let (right_x, right_width) = match class {
            Class::SimpleThin => (SIMPLE_THIN_RIGHT_X, SIMPLE_THIN_RIGHT_WIDTH),
        };
        let y = match class {
            Class::SimpleThin => SIMPLE_THIN_Y,
        };
        let height = match class {
            Class::SimpleThin => SIMPLE_THIN_HEIGHT,
        };
        let color_modifier = match color_class {
            ColorClass::Green => 1,
            ColorClass::Yellow => 2,
            ColorClass::Red => 3,
        };
        let color_y = y + height * color_modifier;

        // BORDER LEFT
        self.sprites.add(Sprite {
            source: Rectangle {
                x: left_x as u16,
                y: y as u16,
                width: left_width as u16,
                height: height as u16,
            },
            // NOTE: + 5.0 but should be like padding ...
            position: Point::new(bounds.x, bounds.y + 5.0),
            scale: (1.0, 2.0),
        });

        // BORDER CENTER
        self.sprites.add(Sprite {
            source: Rectangle {
                x: center_x as u16,
                y: y as u16,
                width: center_width as u16,
                height: height as u16,
            },
            // NOTE: + 5.0 but should be like padding ...
            position: Point::new(bounds.x + SIMPLE_THIN_LEFT_WIDTH as f32, bounds.y + 5.0),
            scale: (bounds.width - SIMPLE_THIN_RIGHT_WIDTH as f32, 2.0),
        });

        // BORDER RIGHT
        self.sprites.add(Sprite {
            source: Rectangle {
                x: right_x as u16,
                y: y as u16,
                width: right_width as u16,
                height: height as u16,
            },
            position: Point::new(
                bounds.x + bounds.width - SIMPLE_THIN_RIGHT_WIDTH as f32,
                // NOTE: + 5.0 but should be like padding ...
                bounds.y + 5.0,
            ),
            scale: (1.0, 2.0),
        });

        let scale_width = (bounds.width - 2.0) * progress;
        // COLOR CENTER
        self.sprites.add(Sprite {
            source: Rectangle {
                x: center_x as u16,
                y: color_y as u16,
                width: center_width as u16,
                height: height as u16,
            },
            // NOTE: + 5.0 but should be like padding ...
            position: Point::new(bounds.x + SIMPLE_THIN_LEFT_WIDTH as f32, bounds.y + 5.0),
            scale: (scale_width, 2.0),
        });
    }
}
