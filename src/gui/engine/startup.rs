use doryen_rs::{DoryenApi, UpdateEvent};

use crate::gui::engine::Engine;

pub struct StartupEngine {
    _mouse_pos: (f32, f32),
}

impl StartupEngine {
    pub fn new() -> Self {
        Self {
            _mouse_pos: (0.0, 0.0),
        }
    }
}

impl Engine for StartupEngine {
    fn get_name(&self) -> &str {
        "STARTUP"
    }

    fn update(
        &mut self,
        _api: &mut dyn DoryenApi,
        _width: i32,
        _height: i32,
    ) -> Option<UpdateEvent> {
        None
    }

    fn render(&mut self, _api: &mut dyn DoryenApi, _width: i32, _height: i32) {}

    fn resize(&mut self, _api: &mut dyn DoryenApi) {}

    fn teardown(&mut self) {}
}
