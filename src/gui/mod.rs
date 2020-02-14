use doryen_rs::{DoryenApi, Engine, UpdateEvent};
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::collections::HashMap;

use crate::color;
use crate::config;
use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::player::Player;
use crate::entity::stuff::Stuff;
use crate::error::RollingError;
use crate::gui::action::Action;
use crate::gui::engine::description::DescriptionEngine;
use crate::gui::engine::startup::StartupEngine;
use crate::gui::engine::world::WorldEngine;
use crate::gui::engine::zone::ZoneEngine;
use crate::gui::engine::Engine as RollingEngine;
use crate::server;
use crate::server::Server;
use crate::tile::zone::Tiles as ZoneTiles;
use crate::util;
use crate::world::level::Level;
use crate::world::socket::ZoneSocket;
use doryen_ui as ui;
use std::error::Error;

pub mod action;
pub mod engine;
pub mod lang;

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
pub const CHAR_TROPICAL_TREE: u16 = 23;
pub const CHAR_DEAD_TREE: u16 = 22;
pub const CHAR_TRUNK: u16 = 146;
pub const CHAR_GEARS: u16 = 128;

pub struct RollingGui {
    engine: Box<dyn RollingEngine>,
    db: PickleDb,
    server: Option<Server>,
    ctx: ui::Context,
    action: action::ActionManager,
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
        let action_conditions: Vec<action::ActionCondition> = vec![
            action::ActionCondition {
                keys: vec!["Space".to_string()],
                engine_id: "ZONE".to_string(),
                to: action::Action::ZoneToWorld,
                wait_while_key: None,
            },
            action::ActionCondition {
                keys: vec!["Escape".to_string()],
                engine_id: "WORLD".to_string(),
                to: action::Action::WorldToZone,
                wait_while_key: Some("Escape".to_string()),
            },
            action::ActionCondition {
                keys: vec!["Escape".to_string()],
                engine_id: "ZONE".to_string(),
                to: action::Action::ZoneToStartup,
                wait_while_key: None,
            },
            action::ActionCondition {
                keys: vec!["Escape".to_string()],
                engine_id: "DESCRIPTION".to_string(),
                to: action::Action::DescriptionToZone,
                wait_while_key: Some("Escape".to_string()),
            },
        ];
        let action = action::ActionManager::new(action_conditions);

        Self {
            engine: Box::new(StartupEngine::new()),
            db: get_db("client.db"),
            ctx: ui::Context::new(),
            width,
            height,
            action,
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
        let stuffs_list = server
            .client
            .get_zone_stuffs(player.world_position.0, player.world_position.1)?;
        let mut stuffs: HashMap<String, Stuff> = HashMap::new();
        for stuff in stuffs_list.into_iter() {
            stuffs.insert(stuff.id.to_string().clone(), stuff);
        }

        // BUILDS
        let builds_list = server
            .client
            .get_zone_builds(player.world_position.0, player.world_position.1)?;
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

        let resume_text = server.client.get_character_resume_texts(&player.id)?;

        Ok(Box::new(ZoneEngine::new(
            server.clone(),
            player,
            characters,
            stuffs,
            builds,
            socket,
            level,
            tiles,
            start_display_map_row_i,
            start_display_map_col_i,
            resume_text,
        )))
    }

    fn setup_world(&self, server: &server::Server) -> Result<Box<dyn RollingEngine>, RollingError> {
        let player = self.create_or_grab_player(server).unwrap().unwrap();

        // Compute display positions (player at center of display)
        let start_display_map_row_i = player.world_position.0 as i32 - (self.height / 2);
        let start_display_map_col_i = player.world_position.1 as i32 - (self.width / 2);

        Ok(Box::new(WorldEngine::new(
            server.clone(),
            player,
            start_display_map_row_i,
            start_display_map_col_i,
        )))
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
    ) -> Result<Option<Player>, Box<dyn Error>> {
        println!("Create or grab character ?");

        if let Some(character_id) = &server.config.character_id {
            println!("Character '{}' locally found", character_id);

            match server.client.get_player(character_id) {
                Ok(player) => {
                    println!("Player found on server");
                    return Ok(Some(player));
                }
                Err(server::client::ClientError::PlayerNotFound { response: _ }) => {
                    println!("Player NOT found on server");
                    return Ok(None);
                }
                Err(client_error) => return Err(Box::new(client_error)),
            }
        }

        println!("No local player found");
        return Ok(None);
    }
    fn build_ui(&mut self) -> Option<action::Action> {
        None
    }
}

impl Engine for RollingGui {
    fn init(&mut self, api: &mut dyn DoryenApi) {
        api.con().register_color("white", color::WHITE);
        api.con().register_color("error", color::RED);
    }
    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        // ui
        ui::update_doryen_input_data(api, &mut self.ctx);
        self.ctx.begin();
        let mut action = None;
        let gui_action = self.build_ui();
        let engine_ui_action =
            self.engine
                .as_mut()
                .build_ui(&mut self.ctx, self.width, self.height);
        self.ctx.end();
        let engine_upd_action = self.engine.as_mut().update(api, self.width, self.height);

        if action.is_none() && gui_action.is_some() {
            action = gui_action;
        }
        if action.is_none() && engine_ui_action.is_some() {
            action = engine_ui_action;
        }
        if action.is_none() && engine_upd_action.is_some() {
            action = engine_upd_action;
        }
        if action.is_none() {
            action = self
                .action
                .resolve(api.input(), self.engine.as_ref().get_name())
        }

        match action {
            Some(action::Action::StartupToZone {
                server_ip,
                server_port,
            }) => {
                println!("Server selected ({}:{})", &server_ip, server_port);
                // TODO: manage error cases
                self.server = Some(self.create_server(&server_ip, server_port).unwrap());
                // TODO: manage error cases
                let player = self
                    .create_or_grab_player(self.server.as_ref().unwrap())
                    .unwrap();
                if let Some(player) = player {
                    // TODO: manage error cases
                    self.engine = self
                        .setup_zone(&self.server.as_ref().unwrap(), player)
                        .unwrap();
                } else {
                    self.engine = Box::new(DescriptionEngine::new(
                        // TODO: manage error cases
                        self.server
                            .as_ref()
                            .unwrap()
                            .client
                            .describe("/_describe/character/create", None, None)
                            .unwrap(),
                        self.server.as_ref().unwrap().clone(),
                    ));
                }
            }
            Some(Action::ZoneToWorld) => {
                println!("Display world map");
                self.engine.teardown();
                // TODO manage error
                self.engine = self.setup_world(&self.server.as_ref().unwrap()).unwrap();
            }
            Some(Action::WorldToZone) => {
                println!("Exit world map");
                self.engine.teardown();
                let player = self
                    .create_or_grab_player(&self.server.as_ref().unwrap())
                    .unwrap()
                    .unwrap();
                self.engine = self
                    .setup_zone(&self.server.as_ref().unwrap(), player)
                    .unwrap();
            }
            Some(Action::DescriptionToZone) => {
                println!("Exit description");
                self.engine.teardown();
                let player = self
                    .create_or_grab_player(&self.server.as_ref().unwrap())
                    .unwrap()
                    .unwrap();
                self.engine = self
                    .setup_zone(&self.server.as_ref().unwrap(), player)
                    .unwrap();
            }
            Some(Action::ZoneToStartup) => {
                println!("Exit zone");
                self.engine.teardown();
                self.engine = self.setup_startup();
            }
            Some(Action::ExitGame) => return Some(UpdateEvent::Exit),
            Some(Action::NewCharacterId { character_id }) => {
                println!("New character {}", &character_id);
                self.server.as_mut().unwrap().config.character_id = Some(character_id);
                self.db
                    .set(
                        format!(
                            "server_{}_{}",
                            self.server.as_ref().unwrap().config.ip,
                            self.server.as_ref().unwrap().config.port
                        )
                        .as_str(),
                        &self.server.as_ref().unwrap().config,
                    )
                    .unwrap();
                let player = self
                    .create_or_grab_player(&self.server.as_ref().unwrap())
                    .unwrap()
                    .unwrap();
                self.engine = self
                    .setup_zone(&self.server.as_ref().unwrap(), player)
                    .unwrap();
            }
            Some(Action::DescriptionToDescription { description }) => {
                println!("Switch description");
                self.engine = Box::new(DescriptionEngine::new(
                    description,
                    self.server.as_ref().unwrap().clone(),
                ));
            }
            Some(Action::ZoneToDescription { url }) => {
                println!("To description");
                // TODO manage error
                let description = self
                    .server
                    .as_ref()
                    .unwrap()
                    .client
                    .describe(url.as_ref(), None, None)
                    .unwrap();
                self.engine = Box::new(DescriptionEngine::new(
                    description,
                    self.server.as_ref().unwrap().clone(),
                ));
            }
            Some(Action::DescriptionToDescriptionGet { url }) => {
                println!("To description");
                // TODO manage error
                let description = self
                    .server
                    .as_ref()
                    .unwrap()
                    .client
                    .describe(url.as_ref(), None, None)
                    .unwrap();
                self.engine = Box::new(DescriptionEngine::new(
                    description,
                    self.server.as_ref().unwrap().clone(),
                ));
            }
            None => {}
        }

        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi) {
        api.con()
            .clear(Some((0, 0, 0, 255)), Some((0, 0, 0, 255)), Some(' ' as u16));
        self.engine.as_mut().render(api, self.width, self.height);
        ui::render_doryen(api.con(), &mut self.ctx);
    }
    fn resize(&mut self, api: &mut dyn DoryenApi) {
        self.engine.as_mut().resize(api);

        self.width = (api.get_screen_size().0 / 18) as i32;
        self.height = (api.get_screen_size().1 / 18) as i32;
        api.con().resize(self.width as u32, self.height as u32);
    }
}
