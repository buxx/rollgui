use doryen_rs::{DoryenApi, UpdateEvent};

pub mod startup;
pub mod zone;
pub mod world;

pub trait Engine {
    fn get_name(&self) -> &str;
    fn update(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) -> Option<UpdateEvent>;
    fn render(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32);
    fn resize(&mut self, api: &mut dyn DoryenApi);
    fn teardown(&mut self);
}
