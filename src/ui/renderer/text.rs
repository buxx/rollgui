use crate::ui::renderer::Renderer;
use crate::ui::widget::text;
use coffee::graphics::{
    self, Color, HorizontalAlignment, Point, Rectangle, Sprite, VerticalAlignment,
};
use coffee::ui::core::{Node, Number, Size, Style};
use coffee::ui::widget::text as coffee_text;

use crate::ui::widget::text::Class;
use std::cell::RefCell;
use std::f32;

const H1_X: u16 = 0;
const H1_Y: u16 = 1460;
const H1_WIDTH: u16 = 600;
const H1_HEIGHT: u16 = 54;

const H2_X: u16 = 0;
const H2_Y: u16 = 1514;
const H2_WIDTH: u16 = 600;
const H2_HEIGHT: u16 = 37;

const PARAGRAPH_X: u16 = 0;
const PARAGRAPH_Y: u16 = 1551;
const PARAGRAPH_WIDTH: u16 = 768;
const PARAGRAPH_HEIGHT: u16 = 37;
const PARAGRAPH_Y_BORDER: u16 = 8;
const PARAGRAPH_PADDING: u16 = 7;

const GRAY1_X: u16 = 180;
const GRAY1_Y: u16 = 1000;
const GRAY2_X: u16 = 180;
const GRAY2_Y: u16 = 1001;
const GRAY3_X: u16 = 180;
const GRAY3_Y: u16 = 1002;

impl text::Renderer for Renderer {
    fn node(&self, style: Style, content: &str, size: f32) -> Node {
        let font = self.font.clone();
        let content = String::from(content);
        let measure = RefCell::new(None);

        Node::with_measure(style, move |bounds| {
            // TODO: Investigate why stretch tries to measure this MANY times
            // with every ancestor's bounds.
            // Bug? Using the library wrong? I should probably open an issue on
            // the stretch repository.
            // I noticed that the first measure is the one that matters in
            // practice. Here, we use a RefCell to store the cached
            // measurement.
            let mut measure = measure.borrow_mut();

            if measure.is_none() {
                let bounds = (
                    match bounds.width {
                        Number::Undefined => f32::INFINITY,
                        Number::Defined(w) => w,
                    },
                    match bounds.height {
                        Number::Undefined => f32::INFINITY,
                        Number::Defined(h) => h,
                    },
                );

                let text = graphics::Text {
                    content: &content,
                    size,
                    bounds,
                    ..graphics::Text::default()
                };

                let (width, height) = font.borrow_mut().measure(text);

                let size = Size { width, height };

                // If the text has no width boundary we avoid caching as the
                // layout engine may just be measuring text in a row.
                if bounds.0 == f32::INFINITY {
                    return size;
                } else {
                    *measure = Some(size);
                }
            }

            measure.unwrap()
        })
    }

    fn draw(
        &mut self,
        bounds: Rectangle<f32>,
        content: &str,
        size: f32,
        color: Color,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
        class: Option<text::Class>,
    ) {
        if let Some(class_) = class {
            match class_ {
                Class::H1 => {
                    self.sprites.add(Sprite {
                        source: Rectangle {
                            x: H1_X,
                            y: H1_Y,
                            width: H1_WIDTH,
                            height: H1_HEIGHT,
                        },
                        position: Point::new(bounds.x, bounds.y),
                        scale: (1.0, 1.0),
                    });
                    self.font.borrow_mut().add(graphics::Text {
                        content,
                        position: Point::new(bounds.x + 15 as f32, bounds.y - 2 as f32),
                        bounds: (bounds.width, bounds.height),
                        color,
                        size,
                        horizontal_alignment,
                        vertical_alignment,
                    });
                    return;
                }
                Class::H2 => {
                    self.sprites.add(Sprite {
                        source: Rectangle {
                            x: H2_X,
                            y: H2_Y,
                            width: H2_WIDTH,
                            height: H2_HEIGHT,
                        },
                        position: Point::new(bounds.x, bounds.y),
                        scale: (1.0, 1.0),
                    });

                    self.font.borrow_mut().add(graphics::Text {
                        content,
                        position: Point::new(bounds.x + 8 as f32, bounds.y + 1 as f32),
                        bounds: (bounds.width, bounds.height),
                        color,
                        size,
                        horizontal_alignment,
                        vertical_alignment,
                    });
                    return;
                }
                Class::Paragraph => {
                    // TOP
                    self.sprites.add(Sprite {
                        source: Rectangle {
                            x: PARAGRAPH_X,
                            y: PARAGRAPH_Y,
                            width: PARAGRAPH_WIDTH,
                            height: PARAGRAPH_Y_BORDER,
                        },
                        position: Point::new(bounds.x, bounds.y),
                        scale: (1.0, 1.0),
                    });
                    // BACKGROUND
                    self.sprites.add(Sprite {
                        source: Rectangle {
                            x: PARAGRAPH_X,
                            y: PARAGRAPH_Y + PARAGRAPH_Y_BORDER,
                            width: PARAGRAPH_WIDTH,
                            height: PARAGRAPH_Y_BORDER,
                        },
                        position: Point::new(bounds.x, bounds.y + PARAGRAPH_Y_BORDER as f32),
                        scale: (
                            1.0,
                            ((bounds.height
                                - PARAGRAPH_Y_BORDER as f32
                                - PARAGRAPH_Y_BORDER as f32
                                + PARAGRAPH_PADDING as f32)
                                / PARAGRAPH_Y_BORDER as f32),
                        ),
                    });
                    // BOTTOM
                    self.sprites.add(Sprite {
                        source: Rectangle {
                            x: PARAGRAPH_X,
                            y: PARAGRAPH_Y + PARAGRAPH_HEIGHT - PARAGRAPH_Y_BORDER,
                            width: PARAGRAPH_WIDTH,
                            height: PARAGRAPH_Y_BORDER,
                        },
                        position: Point::new(
                            bounds.x,
                            bounds.y + bounds.height - PARAGRAPH_Y_BORDER as f32
                                + PARAGRAPH_PADDING as f32,
                        ),
                        scale: (1.0, 1.0),
                    });

                    self.font.borrow_mut().add(graphics::Text {
                        content,
                        position: Point::new(
                            bounds.x as f32 + PARAGRAPH_PADDING as f32,
                            bounds.y as f32 + (PARAGRAPH_PADDING as f32 / 2.0),
                        ),
                        bounds: (bounds.width, bounds.height),
                        color,
                        size,
                        horizontal_alignment,
                        vertical_alignment,
                    });
                    return;
                }
                Class::BgGray1 | Class::BgGray2 | Class::BgGray3 => {
                    let (x, y) = match class_ {
                        Class::BgGray1 => (GRAY1_X, GRAY1_Y),
                        Class::BgGray2 => (GRAY2_X, GRAY2_Y),
                        Class::BgGray3 => (GRAY3_X, GRAY3_Y),
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

                    self.font.borrow_mut().add(graphics::Text {
                        content,
                        position: Point::new(
                            bounds.x as f32 + PARAGRAPH_PADDING as f32,
                            bounds.y as f32 + (PARAGRAPH_PADDING as f32 / 2.0),
                        ),
                        bounds: (bounds.width, bounds.height),
                        color,
                        size,
                        horizontal_alignment,
                        vertical_alignment,
                    });
                    return;
                }
            }
        }

        self.font.borrow_mut().add(graphics::Text {
            content,
            position: Point::new(bounds.x, bounds.y),
            bounds: (bounds.width, bounds.height),
            color,
            size,
            horizontal_alignment,
            vertical_alignment,
        });
    }
}

impl coffee_text::Renderer for Renderer {
    fn node(&self, style: Style, content: &str, size: f32) -> Node {
        let font = self.font.clone();
        let content = String::from(content);
        let measure = RefCell::new(None);

        Node::with_measure(style, move |bounds| {
            // TODO: Investigate why stretch tries to measure this MANY times
            // with every ancestor's bounds.
            // Bug? Using the library wrong? I should probably open an issue on
            // the stretch repository.
            // I noticed that the first measure is the one that matters in
            // practice. Here, we use a RefCell to store the cached
            // measurement.
            let mut measure = measure.borrow_mut();

            if measure.is_none() {
                let bounds = (
                    match bounds.width {
                        Number::Undefined => f32::INFINITY,
                        Number::Defined(w) => w,
                    },
                    match bounds.height {
                        Number::Undefined => f32::INFINITY,
                        Number::Defined(h) => h,
                    },
                );

                let text = graphics::Text {
                    content: &content,
                    size,
                    bounds,
                    ..graphics::Text::default()
                };

                let (width, height) = font.borrow_mut().measure(text);

                let size = Size { width, height };

                // If the text has no width boundary we avoid caching as the
                // layout engine may just be measuring text in a row.
                if bounds.0 == f32::INFINITY {
                    return size;
                } else {
                    *measure = Some(size);
                }
            }

            measure.unwrap()
        })
    }

    fn draw(
        &mut self,
        bounds: Rectangle<f32>,
        content: &str,
        size: f32,
        color: Color,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) {
        self.font.borrow_mut().add(graphics::Text {
            content,
            position: Point::new(bounds.x, bounds.y),
            bounds: (bounds.width, bounds.height),
            color,
            size,
            horizontal_alignment,
            vertical_alignment,
        });
    }
}
