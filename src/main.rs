extern crate doryen_rs;

use doryen_rs::{App, AppOptions};

pub mod color;
pub mod config;
pub mod entity;
pub mod error;
pub mod event;
pub mod gui;
pub mod server;
pub mod tile;
pub mod util;
pub mod world;

const CONSOLE_WIDTH: u32 = 50;
const CONSOLE_HEIGHT: u32 = 40;

fn main() {
    let mut app = App::new(AppOptions {
        console_width: CONSOLE_WIDTH,
        console_height: CONSOLE_HEIGHT,
        screen_width: CONSOLE_WIDTH * 18,
        screen_height: CONSOLE_HEIGHT * 18,
        window_title: "Rolling".to_owned(),
        font_path: "Teeto_K_18x18.png".to_owned(),
        vsync: true,
        fullscreen: false,
        show_cursor: true,
        resizable: true,
        intercept_close_request: false,
    });

    let gui = gui::RollingGui::new(CONSOLE_WIDTH as i32, CONSOLE_HEIGHT as i32);
    app.set_engine(Box::new(gui));
    app.run();
}
