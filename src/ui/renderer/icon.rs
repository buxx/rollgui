use crate::ui::renderer::Renderer;
use crate::ui::widget::icon;
use coffee::graphics::{Point, Rectangle, Sprite};

impl icon::Renderer for Renderer {
    fn draw(&mut self, bounds: Rectangle<f32>, source: Rectangle<u16>) {
        self.sprites.add(Sprite {
            source,
            position: Point::new(bounds.x, bounds.y),
            scale: (1.0, 1.0),
        });
    }
}
