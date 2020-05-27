use coffee::graphics::WindowSettings;
use coffee::ui::UserInterface;
use coffee::Result;

pub mod config;
pub mod engine;
pub mod entity;
pub mod error;
pub mod event;
pub mod game;
pub mod gui;
pub mod input;
pub mod level;
pub mod message;
pub mod server;
pub mod sheet;
pub mod socket;
pub mod tile;
pub mod util;
pub mod world;
pub mod ui;

pub fn main() -> Result<()> {
    <game::MyGame as UserInterface>::run(WindowSettings {
        title: String::from("Coffee"),
        size: (800, 600),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
