use doryen_rs::{DoryenApi, Engine, UpdateEvent};
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::collections::HashMap;

use crate::color;
use crate::config;
use crate::entity::character::Character;
use crate::entity::stuff::Stuff;
use crate::entity::build::Build;
use crate::entity::player::Player;
use crate::server::Server;
use crate::gui::engine::startup::StartupEngine;
use crate::gui::engine::zone::ZoneEngine;
use crate::gui::engine::Engine as RollingEngine;
use crate::gui::engine::world::WorldEngine;
use crate::server;
use crate::tile::zone::Tiles as ZoneTiles;
use crate::tile::world::Tiles as WorldTiles;
use crate::util;
use crate::world::level::Level;
use crate::world::World;
use crate::world::socket::ZoneSocket;
use std::error::Error;
use crate::error::RollingError;

pub mod engine;

pub const CHAR_PLAYER: u16 = 64;
pub const CHAR_CHARACTER: u16 = 1;
pub const CHAR_WATER: u16 = 126;
pub const CHAR_DEEP_WATER: u16 = 247;
pub const CHAR_SAND: u16 = 176;
pub const CHAR_BUSH: u16 = 35;
pub const CHAR_ROCK: u16 = 7;
pub const CHAR_GRASS: u16 = 177;
pub const CHAR_HIGH_GRASS: u16 = 178;
pub const CHAR_TREE: u16 = 5;
pub const CHAR_TROPICAL_TREE: u16 = 226;
pub const CHAR_DEAD_TREE: u16 = 22;
pub const CHAR_TRUNK: u16 = 146;
pub const CHAR_GEARS: u16 = 128;

pub struct RollingGui {
    engine: Box<dyn RollingEngine>,
    db: PickleDb,
    server: Option<Server>,
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
            server: None,
        }
    }

    pub fn setup_startup(&mut self) -> Box<dyn RollingEngine> {
        Box::new(StartupEngine::new())
    }

    pub fn setup_zone(
        &self,
        server: &server::Server,
        player: Player,
    ) -> Result<Box<dyn RollingEngine>, Box<dyn Error>> {
        // TODO: manage error
        let server_tiles_data = server.client.get_tiles_data().unwrap();
        let tiles = ZoneTiles::new(server_tiles_data);
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

        // CHARACTERS
        let all_characters = server
            .client
            .get_zone_characters(player.world_position.0, player.world_position.1)?;
        let mut characters: HashMap<String, Character> = HashMap::new();
        for character in all_characters.into_iter() {
            if player.id != character.id {
                characters.insert(character.id.clone(), character);
            }
        }

        // STUFFS
        let stuffs_list = server.client.get_zone_stuffs(
            player.world_position.0, player.world_position.1
        )?;
        let mut stuffs: HashMap<String, Stuff> = HashMap::new();
        for stuff in stuffs_list.into_iter() {
            stuffs.insert(stuff.id.to_string().clone(), stuff);
        }

        // BUILDS
        let builds_list = server.client.get_zone_builds(
            player.world_position.0, player.world_position.1
        )?;
        let mut builds: HashMap<String, Build> = HashMap::new();
        for build in builds_list.into_iter() {
            builds.insert(build.id.to_string().clone(), build);
        }

        // TODO: https
        let mut socket = ZoneSocket::new(format!(
            "http://{}:{}/zones/{}/{}/events",
            server.config.ip, server.config.port, player.world_position.0, player.world_position.1,
        ));
        socket.connect();

        Ok(
            Box::new(
                ZoneEngine::new(
                    player,
                    characters,
                    stuffs,
                    builds,
                    socket,
                    level,
                    tiles,
                    start_display_map_row_i,
                    start_display_map_col_i,
                )
            )
        )
    }

    fn setup_world(&self, server: &server::Server) -> Result<Box<dyn RollingEngine>, RollingError> {
        let world_source = server.client.get_world_source()?;
        let legend = util::extract_block_from_source("LEGEND", world_source.as_str())?;
        let world_raw = util::extract_block_from_source("GEO", world_source.as_str())?;

        let tiles = WorldTiles::new(legend.as_str())?;
        let world = World::new(world_raw.as_str(), &tiles)?;
        let player = self.create_or_grab_player(server).unwrap();

        // Compute display positions (player at center of display)
        let start_display_map_row_i = player.world_position.0 as i32 - (self.height / 2);
        let start_display_map_col_i = player.world_position.1 as i32 - (self.width / 2);

        Ok(
            Box::new(
                WorldEngine::new(
                    tiles,
                    world,
                    player,
                    start_display_map_row_i,
                    start_display_map_col_i,
                )
            )
        )
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
        &self,
        server: &server::Server,
    ) -> Result<Player, Box<dyn Error>> {
        println!("Create or grab character");

        if let Some(character_id) = &server.config.character_id {
            println!("Character '{}' locally found", character_id);

            match server.client.get_player(character_id) {
                Ok(player) => {
                    println!("Player found on server");
                    return Ok(player);
                }
                Err(server::client::ClientError::PlayerNotFound{ response }) => {
                    println!("Player NOT found on server");
                }
                Err(client_error) => return Err(Box::new(client_error)),
            }
        }

        println!("Character must be created");
        match server.client.create_player("test") {
            Ok(player) => {
                let _character_id = String::from(&player.id);
                println!("Player created with id '{}'", &player.id);
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
            self.server = Some(self.create_server(server_ip, server_port).unwrap());
            // TODO: manage error cases
            let player = self.create_or_grab_player(self.server.as_ref().unwrap()).unwrap();
            // TODO: manage error cases
            self.engine = self.setup_zone(&self.server.as_ref().unwrap(), player).unwrap();
        }

        if input.key("Escape") && self.engine.as_ref().get_name() == "ZONE" {
            println!("Exit zone");
            self.engine.teardown();
            self.engine = self.setup_startup();
        }

        if input.key("Space") && self.engine.as_ref().get_name() == "ZONE" {
            println!("Display world map");
            self.engine.teardown();
            // TODO manage error
            self.engine = self.setup_world(&self.server.as_ref().unwrap()).unwrap();
        }

        if input.key("Escape") && self.engine.as_ref().get_name() == "WORLD" {
            println!("Exit world map");
            self.engine.teardown();
            let player = self.create_or_grab_player(&self.server.as_ref().unwrap()).unwrap();
            self.server.as_mut().unwrap().config.character_id = Some(String::from(&player.id));
            self.db
                .set(
                    format!("server_{}_{}", self.server.as_ref().unwrap().config.ip, self.server.as_ref().unwrap().config.port).as_str(),
                    &self.server.as_ref().unwrap().config,
                )
                .unwrap();
            self.engine = self.setup_zone(&self.server.as_ref().unwrap(), player).unwrap();
        }

        self.engine.as_mut().update(api, self.width, self.height);
        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi) {
        self.engine.as_mut().render(api, self.width, self.height);
    }
    fn resize(&mut self, api: &mut dyn DoryenApi) {
        self.engine.as_mut().resize(api);

        self.width = (api.get_screen_size().0 / 18) as i32;
        self.height = (api.get_screen_size().1 / 18) as i32;
        api.con().resize(self.width as u32, self.height as u32);
    }
}
