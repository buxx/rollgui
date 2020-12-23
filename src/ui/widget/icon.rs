use coffee::graphics::{Point, Rectangle};
use coffee::ui::core::{Element, Hasher, Layout, MouseCursor, Node, Style, Widget};

use std::hash::Hash;

pub enum Class {
    Empty,
    Heart,
    Water,
    Ham,
    AnyWater,
    AnyHam,
    Shield,
    Follower,
    Followed,
    Ok,
    Ko,
    Time,
    Health1,
    Health2,
    Health3,
    Health4,
    Warning,
}

#[derive(Debug)]
pub struct Icon {
    source: Rectangle<u16>,
    style: Style,
}

pub const EMPTY: Rectangle<u16> = Rectangle {
    x: 200,
    y: 80,
    width: 20,
    height: 20,
};

pub const HEART: Rectangle<u16> = Rectangle {
    x: 200,
    y: 100,
    width: 20,
    height: 20,
};

pub const WATER: Rectangle<u16> = Rectangle {
    x: 200,
    y: 120,
    width: 20,
    height: 20,
};

pub const HAM: Rectangle<u16> = Rectangle {
    x: 200,
    y: 140,
    width: 20,
    height: 20,
};

pub const ANY_WATER: Rectangle<u16> = Rectangle {
    x: 200,
    y: 160,
    width: 20,
    height: 20,
};

pub const ANY_HAM: Rectangle<u16> = Rectangle {
    x: 200,
    y: 180,
    width: 20,
    height: 20,
};

pub const SHIELD: Rectangle<u16> = Rectangle {
    x: 200,
    y: 200,
    width: 20,
    height: 20,
};

pub const FOLLOWER: Rectangle<u16> = Rectangle {
    x: 200,
    y: 220,
    width: 20,
    height: 20,
};

pub const FOLLOWED: Rectangle<u16> = Rectangle {
    x: 200,
    y: 240,
    width: 20,
    height: 20,
};

pub const OK: Rectangle<u16> = Rectangle {
    x: 200,
    y: 260,
    width: 20,
    height: 20,
};

pub const KO: Rectangle<u16> = Rectangle {
    x: 200,
    y: 280,
    width: 20,
    height: 20,
};

pub const TIME: Rectangle<u16> = Rectangle {
    x: 200,
    y: 300,
    width: 20,
    height: 20,
};

pub const HEALTH1: Rectangle<u16> = Rectangle {
    x: 200,
    y: 320,
    width: 20,
    height: 20,
};

pub const HEALTH2: Rectangle<u16> = Rectangle {
    x: 200,
    y: 340,
    width: 20,
    height: 20,
};

pub const HEALTH3: Rectangle<u16> = Rectangle {
    x: 200,
    y: 360,
    width: 20,
    height: 20,
};

pub const HEALTH4: Rectangle<u16> = Rectangle {
    x: 200,
    y: 380,
    width: 20,
    height: 20,
};

pub const WARNING: Rectangle<u16> = Rectangle {
    x: 200,
    y: 400,
    width: 20,
    height: 20,
};

fn get_icon_rectangle(icon: Class) -> Rectangle<u16> {
    match icon {
        Class::Empty => EMPTY,
        Class::Heart => HEART,
        Class::Water => WATER,
        Class::Ham => HAM,
        Class::AnyWater => ANY_WATER,
        Class::AnyHam => ANY_HAM,
        Class::Shield => SHIELD,
        Class::Follower => FOLLOWER,
        Class::Followed => FOLLOWED,
        Class::Ok => OK,
        Class::Ko => KO,
        Class::Time => TIME,
        Class::Health1 => HEALTH1,
        Class::Health2 => HEALTH2,
        Class::Health3 => HEALTH3,
        Class::Health4 => HEALTH4,
        Class::Warning => WARNING,
    }
}

impl Icon {
    pub fn new(class: Class) -> Self {
        let source = get_icon_rectangle(class);
        Icon {
            source,
            style: Style::default().fill_width(),
        }
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Icon
where
    Renderer: self::Renderer,
{
    fn node(&self, _renderer: &Renderer) -> Node {
        Node::new(self.style.height(self.source.height as u32))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        renderer.draw(layout.bounds(), self.source);
        MouseCursor::OutOfBounds
    }

    fn hash(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

pub trait Renderer {
    fn draw(&mut self, bounds: Rectangle<f32>, source: Rectangle<u16>);
}

impl<'a, Message, Renderer> From<Icon> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(icon: Icon) -> Element<'a, Message, Renderer> {
        Element::new(icon)
    }
}
