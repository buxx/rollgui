use crate::engine::description::DescriptionEngine;
use crate::engine::exit::ExitEngine;
use crate::engine::startup::StartupEngine;
use crate::engine::world::WorldEngine;
use crate::engine::zone::ZoneEngine;
use crate::engine::Engine;
use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::player::Player;
use crate::entity::resource::Resource;
use crate::entity::stuff::Stuff;
use crate::input::MyGameInput;
use crate::level::Level;
use crate::message::{MainMessage, Message};
use crate::socket::ZoneSocket;
use crate::tile::zone::Tiles as ZoneTiles;
use crate::ui::renderer::Renderer;
use crate::ui::{Column, Element};
use crate::{config, event, server, util};
use coffee::graphics::{Color, Frame, HorizontalAlignment, VerticalAlignment, Window};
use coffee::load::{loading_screen, Task};
use coffee::ui::{Align, Justify, Text, UserInterface};
use coffee::{graphics, Game, Timer};
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::collections::HashMap;
use std::error::Error;

// TODO: dynamic from server (and tilesheet)
pub const TILE_WIDTH: i16 = 16;
pub const TILE_HEIGHT: i16 = 16;

pub struct MyGame {
    engine: Box<dyn Engine>,
    tile_sheet_image: graphics::Image,
    db: PickleDb,
    server: Option<server::Server>,
    player: Option<Player>,
    exit_requested: bool,
    pending_action: Option<MainMessage>,
    loading_displayed: bool,
}

fn get_db(db_file_path: &str) -> PickleDb {
    if let Ok(db) = PickleDb::load_json(&db_file_path, PickleDbDumpPolicy::AutoDump) {
        return db;
    }

    PickleDb::new_json(&db_file_path, PickleDbDumpPolicy::AutoDump)
}

impl MyGame {
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

    fn setup_startup_to_zone_engine(&mut self, server_ip: String, server_port: u16) {
        // FIXME BS: manage error cases
        self.server = Some(self.create_server(server_ip, server_port).unwrap());
        let server = self.server.as_ref().unwrap().clone();

        // FIXME BS: manage error cases
        if let Some(player) = self.create_player().unwrap() {
            self.player = Some(player);
            self.setup_zone_engine();
            return;
        }

        if let Some(character_id) = &server.config.character_id {
            println!("Maybe character is dead ?");
            // FIXME: manage error case
            if server.client.player_is_dead(&character_id).unwrap() {
                println!("Yes, it is dead");
                self.engine = Box::new(DescriptionEngine::new(
                    // TODO: manage error cases
                    server
                        .client
                        .describe(
                            format!("/character/{}/post_mortem", character_id).as_str(),
                            None,
                            None,
                        )
                        .unwrap(),
                    server.clone(),
                    None,
                ));
            }
        }

        self.engine = Box::new(DescriptionEngine::new(
            // FIXME: manage error cases
            server
                .client
                .describe("/_describe/character/create", None, None)
                .unwrap(),
            server.clone(),
            None,
        ));
    }

    fn create_server(
        &self,
        server_ip: String,
        server_port: u16,
    ) -> Result<server::Server, Box<dyn Error>> {
        let server_config = self.get_server_config(&server_ip, server_port);
        let client = server::client::Client::new(&server_ip, server_port);
        Ok(server::Server::new(client, server_config)?)
    }

    fn create_player(&self) -> Result<Option<Player>, Box<dyn Error>> {
        // Server must exist at this step
        let server = self.server.as_ref().unwrap();

        println!("Try to create Player with local data?");
        if let Some(character_id) = &server.config.character_id {
            println!("Character '{}' locally found", character_id);
            return match server.client.get_player(character_id) {
                Ok(player) => {
                    println!("Player found on server");
                    Ok(Some(player))
                }
                Err(server::client::ClientError::PlayerNotFound { response: _ }) => {
                    println!("Player NOT found on server");
                    Ok(None)
                }
                Err(client_error) => Err(Box::new(client_error)),
            };
        }

        println!("No local player found");
        return Ok(None);
    }

    fn setup_zone_engine(&mut self) {
        // Player and Server must exist at this step
        let server = self.server.as_ref().unwrap();
        let player = self.player.as_ref().unwrap();

        let tile_sheet_image = self.tile_sheet_image.clone();
        // FIXME BS: manage error case
        let server_tiles_data = server.client.get_tiles_data().unwrap();
        let tiles = ZoneTiles::new(server_tiles_data);
        let tile_width: i16 = TILE_WIDTH;
        let tile_height: i16 = TILE_HEIGHT;

        // FIXME BS: manage error
        let zone_data = server
            .client
            .get_zone_data(player.world_position.0, player.world_position.1)
            .unwrap();
        // FIXME BS: manage error
        let zone_raw = zone_data["raw_source"].as_str().unwrap();
        let zone_raw = util::extract_block_from_source(util::BLOCK_GEO, zone_raw).unwrap();
        let world_tile_type_id = self.server.as_ref().unwrap().world.rows
            [player.world_position.0 as usize]
            .cols[player.world_position.1 as usize]
            .clone();
        let level = Level::new(&zone_raw, &tiles, world_tile_type_id).unwrap();

        // TODO: https
        let mut socket = ZoneSocket::new(format!(
            "http://{}:{}/zones/{}/{}/events",
            server.config.ip, server.config.port, player.world_position.0, player.world_position.1,
        ));
        socket.connect();
        socket.send(event::ZoneEvent {
            event_type_name: String::from(event::CLIENT_REQUIRE_AROUND),
            event_type: event::ZoneEventType::ClientRequireAround {
                zone_row_i: player.position.0 as i32,
                zone_col_i: player.position.1 as i32,
                character_id: String::from(player.id.as_str()),
            },
        });
        // FIXME: Manage errors
        let resume_text = server
            .client
            .get_character_resume_texts(&player.id)
            .unwrap();

        // CHARACTERS
        // FIXME: Manage errors
        let all_characters = server
            .client
            .get_zone_characters(player.world_position.0, player.world_position.1)
            .unwrap();
        let mut characters: HashMap<String, Character> = HashMap::new();
        for character in all_characters.into_iter() {
            if player.id != character.id {
                characters.insert(character.id.clone(), character);
            }
        }

        // STUFFS
        // FIXME: Manage errors
        let stuffs_list = server
            .client
            .get_zone_stuffs(player.world_position.0, player.world_position.1)
            .unwrap();
        let mut stuffs: HashMap<String, Stuff> = HashMap::new();
        for stuff in stuffs_list.into_iter() {
            stuffs.insert(stuff.id.to_string().clone(), stuff);
        }

        // RESOURCES
        // FIXME: Manage errors
        let resources: Vec<Resource> = server
            .client
            .get_zone_resources(player.world_position.0, player.world_position.1)
            .unwrap();

        // BUILDS
        // FIXME: Manage errors
        let builds_list = server
            .client
            .get_zone_builds(player.world_position.0, player.world_position.1)
            .unwrap();
        let mut builds: HashMap<String, Build> = HashMap::new();
        for build in builds_list.into_iter() {
            builds.insert(build.id.to_string().clone(), build);
        }

        self.engine = Box::new(ZoneEngine::new(
            tiles,
            tile_sheet_image,
            tile_width,
            tile_height,
            player.clone(),
            server.clone(),
            level,
            socket,
            resume_text,
            characters,
            stuffs,
            resources,
            builds,
        ));
    }

    fn proceed_main_message(&mut self, main_message: MainMessage) {
        self.pending_action = Some(main_message);
    }
}

impl Game for MyGame {
    type Input = MyGameInput;
    type LoadingScreen = loading_screen::ProgressBar;
    const TICKS_PER_SECOND: u16 = 60;

    fn load(_window: &Window) -> Task<MyGame> {
        graphics::Image::load("resources/tilesheet.png").map(|image| MyGame {
            engine: Box::new(StartupEngine::new()),
            tile_sheet_image: image,
            db: get_db("client.db"),
            server: None,
            player: None,
            exit_requested: false,
            pending_action: None,
            loading_displayed: false,
        })
    }

    fn interact(&mut self, input: &mut MyGameInput, window: &mut Window) {
        match self.engine.interact(input, window) {
            Some(main_message) => self.proceed_main_message(main_message),
            None => {}
        }
    }

    fn update(&mut self, window: &Window) {
        if self.loading_displayed {
            let main_message = self.pending_action.as_ref().unwrap().clone();
            self.pending_action = None;
            self.loading_displayed = false;

            match main_message {
                MainMessage::StartupToZone {
                    server_ip,
                    server_port,
                } => {
                    self.setup_startup_to_zone_engine(server_ip.clone(), server_port);
                }
                MainMessage::ToDescriptionWithDescription {
                    description,
                    back_url,
                } => {
                    self.engine = Box::new(DescriptionEngine::new(
                        description.clone(),
                        self.server.as_ref().unwrap().clone(),
                        back_url.clone(),
                    ));
                }
                MainMessage::NewCharacterId { character_id } => {
                    println!("New character {}", &character_id);
                    self.server.as_mut().unwrap().config.character_id = Some(character_id.clone());
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
                    self.setup_startup_to_zone_engine(
                        self.server.as_ref().unwrap().config.ip.clone(),
                        self.server.as_ref().unwrap().config.port,
                    );
                }
                MainMessage::ToDescriptionWithUrl { url, back_url } => {
                    // FIXME: manage errors
                    let description = self
                        .server
                        .as_ref()
                        .unwrap()
                        .client
                        .describe(&url, None, None)
                        .unwrap();
                    self.engine = Box::new(DescriptionEngine::new(
                        description,
                        self.server.as_ref().unwrap().clone(),
                        back_url.clone(),
                    ));
                }
                MainMessage::DescriptionToZone => {
                    // FIXME: manage errors
                    self.player = self.create_player().unwrap(); // must succeed ...
                    self.setup_zone_engine();
                }
                MainMessage::ToStartup => {
                    self.engine = Box::new(StartupEngine::new());
                }
                MainMessage::ToExit => {
                    self.engine = Box::new(ExitEngine::new());
                }
                MainMessage::ExitRequested => self.exit_requested = true,
                MainMessage::ToWorld => {
                    self.engine = Box::new(WorldEngine::new(
                        self.server.as_ref().unwrap().clone(),
                        self.tile_sheet_image.clone(),
                        self.player.as_ref().unwrap().clone(),
                    ));
                }
            }
        }

        match self.engine.update(window) {
            Some(main_message) => self.proceed_main_message(main_message),
            None => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        if self.pending_action.is_some() {
            frame.clear(Color::BLACK);
        } else {
            self.engine.draw(frame, timer)
        }
    }

    fn is_finished(&self) -> bool {
        self.exit_requested
    }
}

impl UserInterface for MyGame {
    type Message = Message;
    type Renderer = Renderer;

    fn react(&mut self, event: Message, window: &mut Window) {
        match self.engine.as_mut().react(event, window) {
            Some(main_message) => self.proceed_main_message(main_message),
            None => {}
        }
    }

    fn layout(&mut self, window: &Window) -> Element {
        if self.pending_action.is_some() {
            self.loading_displayed = true;
            Column::new()
                .width(window.width() as u32)
                .height(window.height() as u32)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .spacing(20)
                .push(
                    Text::new("Chargement ...")
                        .size(50)
                        .height(60)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .into()
        } else {
            self.engine.layout(window)
        }
    }
}
