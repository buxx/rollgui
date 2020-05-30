use coffee::ui::{core as coffee_core, widget as coffee_widget};
use crate::message;

pub mod renderer;
pub mod widget;

pub type Element<'a> = coffee_core::Element<'a, message::Message, renderer::Renderer>;
pub type Column<'a> = coffee_widget::Column<'a, message::Message, renderer::Renderer>;
pub type Row<'a> = coffee_widget::Row<'a, message::Message, renderer::Renderer>;
