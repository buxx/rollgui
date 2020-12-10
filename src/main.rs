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
pub mod ui;
pub mod util;
pub mod world;

pub fn main() -> Result<()> {
    let _guard = sentry::init("https://7f725b87c5494a66983f69228fc9fd3c@o433154.ingest.sentry.io/5551646");

    <game::MyGame as UserInterface>::run(WindowSettings {
        title: String::from("Coffee"),
        size: (1024, 768),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
