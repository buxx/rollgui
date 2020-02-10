use doryen_rs::{DoryenApi, UpdateEvent};
use doryen_ui as ui;

use crate::gui::action;

pub mod description;
pub mod startup;
pub mod world;
pub mod zone;

pub trait Engine {
    fn get_name(&self) -> &str;
    fn update(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) -> Option<UpdateEvent>;
    fn render(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32);
    fn resize(&mut self, api: &mut dyn DoryenApi);
    fn build_ui(
        &mut self,
        ctx: &mut ui::Context,
        width: i32,
        height: i32,
    ) -> Option<action::Action>;
    fn teardown(&mut self);
}
