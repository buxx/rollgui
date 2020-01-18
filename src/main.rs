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
pub mod zone;

const CONSOLE_WIDTH: u32 = 80;
const CONSOLE_HEIGHT: u32 = 45;

fn main() {
    let mut app = App::new(AppOptions {
        console_width: CONSOLE_WIDTH,
        console_height: CONSOLE_HEIGHT,
        screen_width: CONSOLE_WIDTH * 8,
        screen_height: CONSOLE_HEIGHT * 8,
        window_title: "my roguelike".to_owned(),
        font_path: "terminal_8x8.png".to_owned(),
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
