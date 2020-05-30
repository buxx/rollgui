use coffee::ui::{core, widget};

pub mod renderer;

pub type Element<'a, Message> = self::core::Element<'a, Message, renderer::Renderer>;
pub type Column<'a, Message> = widget::Column<'a, Message, renderer::Renderer>;
pub type Row<'a, Message> = widget::Row<'a, Message, renderer::Renderer>;
