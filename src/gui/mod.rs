use doryen_rs::{DoryenApi, Engine, UpdateEvent};
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::collections::HashMap;

use crate::color;
use crate::config;
use crate::entity::character::Character;
use crate::entity::player::Player;
use crate::error::RollingError;
use crate::gui::engine::startup::StartupEngine;
use crate::gui::engine::zone::ZoneEngine;
use crate::gui::engine::Engine as RollingEngine;
use crate::server;
use crate::tile::Tiles;
use crate::util;
use crate::zone::level::Level;
use crate::zone::socket::ZoneSocket;
use std::error::Error;

pub mod engine;

pub struct RollingGui {
    engine: Box<dyn RollingEngine>,
    db: PickleDb,
    pub width: i32,
    pub height: i32,
}

fn get_db(db_file_path: &str) -> PickleDb {
    if let Ok(db) = PickleDb::load_json(&db_file_path, PickleDbDumpPolicy::AutoDump) {
        return db;
    }

    PickleDb::new_json(&db_file_path, PickleDbDumpPolicy::AutoDump)
}

impl RollingGui {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            engine: Box::new(StartupEngine::new()),
            db: get_db("client.db"),
            width,
            height,
        }
    }

    pub fn setup_startup(&mut self) {
        self.engine = Box::new(StartupEngine::new());
    }

    pub fn setup_zone(
        &mut self,
        server: server::Server,
        player: Player,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: manage error
        let server_tiles_data = server.client.get_tiles_data().unwrap();
        let tiles = Tiles::new(server_tiles_data);
        // TODO: manage error
        let zone_data = server
            .client
            .get_zone_data(player.world_position.0, player.world_position.1)
            .unwrap();
        // TODO: manage error
        let zone_raw = zone_data["raw_source"].as_str().unwrap();
        let zone_raw = util::extract_block_from_source(util::BLOCK_GEO, zone_raw)?;

        let level = Level::new(&zone_raw, &tiles)?;

        // Compute display positions (player at center of display)
        let start_display_map_row_i = player.position.0 as i32 - (self.height / 2);
        let start_display_map_col_i = player.position.1 as i32 - (self.width / 2);

        let all_characters = server
            .client
            .get_zone_characters(player.world_position.0, player.world_position.1)?;
        let mut characters: HashMap<String, Character> = HashMap::new();
        for character in all_characters.into_iter() {
            if player.id != character.id {
                characters.insert(character.id.clone(), character);
            }
        }

        // TODO: https
        let mut socket = ZoneSocket::new(format!(
            "http://{}:{}/zones/{}/{}/events",
            server.config.ip, server.config.port, player.world_position.0, player.world_position.1,
        ));
        socket.connect();

        self.engine = Box::new(ZoneEngine::new(
            server,
            player,
            characters,
            socket,
            level,
            tiles,
            start_display_map_row_i,
            start_display_map_col_i,
        ));
        Ok(())
    }

    fn get_server_config(&self, server_ip: &str, server_port: u16) -> config::ServerConfig {
        if let Some(server_config) = self
            .db
            .get::<config::ServerConfig>(format!("server_{}_{}", server_ip, server_port).as_str())
        {
            return server_config;
        }

        config::ServerConfig {
            ip: server_ip.to_string(),
            port: server_port,
            character_id: None,
        }
    }

    fn create_server(
        &mut self,
        server_ip: &str,
        server_port: u16,
    ) -> Result<server::Server, Box<dyn Error>> {
        let server_config = self.get_server_config(server_ip, server_port);
        let client = server::client::Client::new(server_ip, server_port);

        Ok(server::Server::new(client, server_config)?)
    }

    fn create_or_grab_player(
        &mut self,
        server: &mut server::Server,
    ) -> Result<Player, Box<dyn Error>> {
        println!("Create or grab character");

        if let Some(character_id) = &server.config.character_id {
            println!("Character '{}' locally found", character_id);

            match server.client.get_player(character_id) {
                Ok(player) => {
                    println!("Player found on server");
                    return Ok(player);
                }
                Err(server::client::ClientError::PlayerNotFound) => {
                    println!("Player NOT found on server");
                }
                Err(client_error) => return Err(Box::new(client_error)),
            }
        }

        println!("Character must be created");
        match server.client.create_player("test") {
            Ok(player) => {
                let character_id = String::from(&player.id);
                println!("Player created with id '{}'", &player.id);
                server.config.character_id = Some(String::from(&player.id));
                self.db
                    .set(
                        format!("server_{}_{}", server.config.ip, server.config.port).as_str(),
                        &server.config,
                    )
                    .unwrap();
                return Ok(player);
            }
            Err(client_error) => return Err(Box::new(client_error)),
        }
    }
}

impl Engine for RollingGui {
    fn init(&mut self, api: &mut dyn DoryenApi) {
        api.con().register_color("white", color::WHITE);
    }
    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        api.con()
            .clear(Some(color::BLACK), Some(color::BLACK), Some(' ' as u16));
        let input = api.input();

        if input.key("Enter") && self.engine.as_ref().get_name() != "ZONE" {
            // TODO: manage setup zone fail (with gui message)
            let server_ip = "127.0.0.1";
            let server_port: u16 = 5000;
            println!("Server selected ({}:{})", &server_ip, server_port);
            // TODO: manage error cases
            let mut server = self.create_server(server_ip, server_port).unwrap();
            // TODO: manage error cases
            let player = self.create_or_grab_player(&mut server).unwrap();
            // TODO: manage error cases
            self.setup_zone(server, player).unwrap();
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
