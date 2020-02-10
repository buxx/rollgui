use doryen_rs::{DoryenApi, Image, UpdateEvent};
use doryen_ui as ui;

use crate::gui::action;
use crate::gui::engine::Engine;

pub struct StartupEngine {
    image: Image,
}

impl StartupEngine {
    pub fn new() -> Self {
        Self {
            image: Image::new("title.png"),
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

    fn build_ui(
        &mut self,
        ctx: &mut ui::Context,
        width: i32,
        height: i32,
    ) -> Option<action::Action> {
        ctx.frame_window_begin("main_menu", "Main menu", 10, 10, 32, 32);
        ctx.frame_begin("menu", "Rejoindre un univers", 32, 20)
            .margin(2);

        let buttons = vec![
            ("server1", "127.0.0.1", 5000, "Serveur local"),
            ("server2", "s2.bux.fr", 7431, "s2.bux.fr"),
        ];
        ctx.label("Rejoindre un univers")
            .align(ui::TextAlign::Center);
        ctx.label("").align(ui::TextAlign::Center);

        for button in buttons.iter() {
            let (id, host, port, label) = button;

            if ctx.button(id, label).align(ui::TextAlign::Center).pressed() {
                return Some(action::Action::StartupToZone {
                    server_ip: host.to_string(),
                    server_port: *port as u16,
                });
            }
        }

        ctx.label("").align(ui::TextAlign::Center);
        if ctx
            .button("exit", "Quitter")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ExitGame);
        }

        None
    }

    fn teardown(&mut self) {}
}
