use doryen_rs::{DoryenApi, Image, TextAlign};
use doryen_ui as ui;

use crate::color;
use crate::gui::action;
use crate::gui::engine::Engine;
use crate::server::Server;

pub struct ConfirmExitEngine {
    server: Server,
    image: Image,
    loading: bool,
    loading_displayed: bool,
    loading_closure: Option<Box<dyn Fn() -> action::Action>>,
}

impl ConfirmExitEngine {
    pub fn new(server: Server) -> Self {
        Self {
            server,
            image: Image::new("exit.png"),
            loading: false,
            loading_displayed: false,
            loading_closure: None,
        }
    }
}

impl Engine for ConfirmExitEngine {
    fn get_name(&self) -> &str {
        "CONFIRM_EXIT"
    }

    fn update(
        &mut self,
        _api: &mut dyn DoryenApi,
        _width: i32,
        _height: i32,
    ) -> Option<action::Action> {
        None
    }

    fn render(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) {
        api.con()
            .clear(Some(color::BLACK), Some(color::BLACK), Some(' ' as u16));
        if self.loading {
            api.con()
                .clear(Some(color::BLACK), Some(color::BLACK), Some(' ' as u16));
            api.con().print(
                width / 2,
                height / 2,
                "Chargement ...",
                TextAlign::Center,
                Some(color::WHITE),
                Some(color::BLACK),
            );
            return;
        }
        let dx = (width / 2) - (self.image.width() as i32 / 4);
        self.image.blit_2x(api.con(), dx, 1, 0, 0, None, None, None);
    }

    fn resize(&mut self, _api: &mut dyn DoryenApi) {}

    fn build_ui(
        &mut self,
        ctx: &mut ui::Context,
        width: i32,
        _height: i32,
    ) -> Option<action::Action> {
        if self.loading {
            if self.loading_displayed {
                return Some(self.loading_closure.as_ref().unwrap()());
            }
            self.loading_displayed = true;
            return None;
        }

        ctx.frame_window_begin("main_menu", "Main menu", (width / 2) - (32/2), 27, 32, 3);

        ctx.label("Voulez-vous vraiment quitter ?");
        if ctx.button("stay", "Annuler").align(ui::TextAlign::Center).pressed() {
            self.loading = true;
            let ip = self.server.config.ip.clone();
            let port = self.server.config.port;
            self.loading_closure = Some(Box::new(move || {
                return action::Action::StartupToZone {
                    server_ip: ip.clone(),
                    server_port: port,
                };
            }));
        }

        if ctx.button("exit", "Quitter").align(ui::TextAlign::Center).pressed() {
            return Some(action::Action::ToStartup);
        }

        None
    }

    fn teardown(&mut self) {}
}
