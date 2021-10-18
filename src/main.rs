use crate::util::get_conf;
use coffee::graphics::WindowSettings;
use coffee::ui::UserInterface;
use coffee::Result;
use structopt::StructOpt;

pub mod args;
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
    let opt = args::Opt::from_args();
    let conf = get_conf(&opt.config_file_path);
    match conf
        .get_from(Some("debug"), "enable_bug_report")
        .unwrap_or("false")
    {
        "true" | "True" | "1" => {
            println!("Enable bug report (configurable in config.ini)");
            let _guard = sentry::init(
                "https://7f725b87c5494a66983f69228fc9fd3c@o433154.ingest.sentry.io/5551646",
            );
        }
        _ => {}
    };
    let title = conf.get_from(Some("design"), "title").unwrap_or("Rolling");

    <game::MyGame as UserInterface>::run(WindowSettings {
        title: String::from(title),
        size: (1000, 700),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
