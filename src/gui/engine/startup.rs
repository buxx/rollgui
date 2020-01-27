use doryen_rs::{DoryenApi, UpdateEvent, Image};

use crate::gui::engine::Engine;

pub struct StartupEngine {
    image: Image,
}

impl StartupEngine {
    pub fn new() -> Self {
        Self {
            image: Image::new("island.png"),
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

    fn render(&mut self, api: &mut dyn DoryenApi, _width: i32, _height: i32) {
        self.image.blit_2x(api.con(), 0, 0, 0, 0, None, None, None);
    }

    fn resize(&mut self, _api: &mut dyn DoryenApi) {}

    fn teardown(&mut self) {}
}
