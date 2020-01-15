use doryen_rs::{DoryenApi, Engine, UpdateEvent};
use pickledb::{PickleDb, PickleDbDumpPolicy};
use reqwest::Client;

use std::path::Path;
use crate::color;
use crate::util;
use std::error::Error;
use crate::tile::{Tiles};
use crate::zone::level::{Level};
use crate::entity::player::{Player};
use crate::gui::engine::{Engine as RollingEngine};
use crate::gui::engine::zone::{ZoneEngine};
use crate::gui::engine::startup::{StartupEngine};
use crate::zone::socket::{ZoneSocket};
use crate::config;
use std::collections::HashMap;

pub mod engine;

pub struct RollingGui {
    engine: Box<dyn RollingEngine>,
    db: PickleDb,
    pub width: i32,
    pub height: i32,
}

impl RollingGui {
    pub fn new(width: i32, height: i32) -> Self {
        // TODO: client.db in home
        let db = PickleDb::new_json("client.db", PickleDbDumpPolicy::AutoDump);

        Self {
            engine: Box::new(StartupEngine::new()),
            db,
            width,
            height,
        }
    }

    pub fn setup_startup(&mut self) {
        self.engine = Box::new(StartupEngine::new());
    }

    pub fn setup_zone(&mut self, world_row_i: i32, world_col_i: i32) -> Result<(), Box<dyn Error>> {
        // TODO: from server
        let tiles= Tiles::new();
        // TODO: from server
        let zone_file_content = util::get_file_content(
            Path::new("static/zone_test.txt")
        )?;
        let zone_raw = util::extract_block_from_source(
            util::BLOCK_GEO,
            &zone_file_content,
        )?;
        let level = Level::new(&zone_raw, &tiles)?;

        // TODO: from server
        let player_row_i = 0;
        let player_col_i = 1;

        // Compute display positions (player at center of display)
        let start_display_map_row_i = player_row_i - (self.height / 2);
        let start_display_map_col_i = player_col_i - (self.width / 2);

        let player = Player::new((player_row_i, player_col_i));

        // TODO: https
        let mut socket = ZoneSocket::new(
            format!(
                "http://{}/zones/{}/{}/events",
                "127.0.0.1",
                world_row_i,
                world_col_i,
            )
        );
        socket.connect();

        self.engine = Box::new(
            ZoneEngine::new(
                player,
                socket,
                level,
                tiles,
                start_display_map_row_i,
                start_display_map_col_i,
            )
        );
        Ok(())
    }

    fn create_or_grab_character(&self, server_ip: String, server_port: u16) {
        let server_config = self.db.get::<config::ServerConfig>(
            format!("server_{}", server_ip.as_str()).as_str()
        );

        if server_config.is_some() {
            dbg!(&server_config);
        } else {
            println!("Character must be created");


            let mut map = HashMap::new();
            map.insert("lang", "rust");
            map.insert("body", "json");

            let client = reqwest::blocking::Client::new();
            let res = client.post("http://httpbin.org/post")
                .body("the exact body that is sent")
                .send()
                .unwrap();

            dbg!(res);
        }
    }
}

impl Engine for RollingGui {
    fn init(&mut self, api: &mut dyn DoryenApi) {
        api.con().register_color("white", color::WHITE);
    }
    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        api.con().clear(Some(color::BLACK), Some(color::BLACK), Some(' ' as u16));
        let input = api.input();

        if input.key("Enter") && self.engine.as_ref().get_name() != "ZONE" {
            // TODO: manage setup zone fail (with gui message)
            let server_ip = String::from("127.0.0.1");
            let server_port: u16 = 5000;
            let character = self.create_or_grab_character(server_ip, server_port);
            self.setup_zone(0, 0).unwrap();
        }

        if input.key("Escape") && self.engine.as_ref().get_name() == "ZONE" {
            self.engine.teardown();
            self.setup_startup();
        }

        self.engine.as_mut().update(api, self.width, self.height);
        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi) {
        self.engine.as_mut().render(api, self.width, self.height);
    }
    fn resize(&mut self, api: &mut dyn DoryenApi) {
        self.engine.as_mut().resize(api);

        self.width = (api.get_screen_size().0 / 8) as i32;
        self.height = (api.get_screen_size().1 / 8) as i32;
        api.con().resize(self.width as u32, self.height as u32);
    }
}
