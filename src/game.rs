use crate::args;
use crate::engine::description::DescriptionEngine;
use crate::engine::exit::ExitEngine;
use crate::engine::login::LoginEngine;
use crate::engine::upgrade::UpgradeEngine;
use crate::engine::world::WorldEngine;
use crate::engine::zone::ZoneEngine;
use crate::engine::Engine;
use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::corpse::AnimatedCorpse;
use crate::entity::player::Player;
use crate::entity::resource::Resource;
use crate::entity::stuff::Stuff;
use crate::gui::lang::model::RequestClicks;
use crate::input::MyGameInput;
use crate::level::Level;
use crate::message::{MainMessage, Message};
use crate::sheet::TileSheet;
use crate::socket::ZoneSocket;
use crate::tile::zone::Tiles as ZoneTiles;
use crate::ui::renderer::Renderer;
use crate::ui::widget::text::Text;
use crate::ui::{Column, Element};
use crate::util::get_conf;
use crate::{event, server, util};
use glob::glob;
use rand::seq::SliceRandom;

use coffee::graphics::{Color, Frame, HorizontalAlignment, VerticalAlignment, Window};
use coffee::load::{loading_screen, Task};
use coffee::ui::{Align, Image, Justify, UserInterface};
use coffee::{graphics, Game, Timer};
use dialog::DialogBox;
use ini::Ini;
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::collections::HashMap;
use std::error::Error;
use std::process::exit;
use std::thread;
use std::time::SystemTime;
use structopt::StructOpt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// TODO: dynamic from server (and tilesheet)
pub const TILE_WIDTH: i16 = 32;
pub const TILE_HEIGHT: i16 = 32;

pub const TARGET_FRAME_DURATION_MS: u64 = 16; // target is ~60fps

pub struct MyGame {
    conf: Ini,
    engine: Option<Box<dyn Engine>>,
    tile_sheet_image: graphics::Image,
    db: PickleDb,
    server: server::Server,
    player: Option<Player>,
    exit_requested: bool,
    pending_action: Option<MainMessage>,
    loading_displayed: bool,
    last_tick: SystemTime,
    pending_illustration: Option<String>,
    illustration: Option<graphics::Image>,
    illustration_bg: Option<graphics::Image>,
    pending_home_image: Option<String>,
    home_image: Option<graphics::Image>,
    loading_image_to_set: bool,
    loading_image: Option<graphics::Image>,
}

fn get_db(db_file_path: &str) -> PickleDb {
    if let Ok(db) = PickleDb::load_json(&db_file_path, PickleDbDumpPolicy::AutoDump) {
        return db;
    }

    PickleDb::new_json(&db_file_path, PickleDbDumpPolicy::AutoDump)
}

impl MyGame {
    fn get_server_last_username(&self, address: server::ServerAddress) -> String {
        if let Some(last_username) = self
            .db
            .get::<String>(format!("server_{}_{}", address.host, address.port).as_str())
        {
            return last_username;
        }

        "".to_string()
    }

    fn set_server_last_username(&mut self) {
        self.db
            .set(
                format!(
                    "server_{}_{}",
                    self.server.address.host, self.server.address.port
                )
                .as_str(),
                &self.server.client.credentials.0,
            )
            .unwrap();
    }

    fn setup_startup_to_zone_engine(&mut self, request_clicks: Option<RequestClicks>) {
        println!("setup_startup_to_zone_engine");
        let server = self.server.clone();

        // FIXME BS: manage error cases
        if let Some(player) = self.create_player().unwrap() {
            self.player = Some(player);
            self.setup_zone_engine(request_clicks);
            return;
        }

        if let Some(character_id) = server.character_id.clone() {
            println!("Maybe character is dead ?");
            // FIXME: manage error case
            if server.client.player_is_dead(&character_id).unwrap() {
                println!("Yes, it is dead");
                let description = server
                    .client
                    .describe(
                        format!("/character/{}/post_mortem", character_id).as_str(),
                        None,
                        None,
                    )
                    .unwrap();
                self.setup_no_home_image();
                self.engine = Some(Box::new(DescriptionEngine::new(
                    None,
                    // TODO: manage error cases
                    description.clone(),
                    server.client.clone(),
                    None,
                    true,
                    TileSheet::new(self.tile_sheet_image.clone(), TILE_WIDTH, TILE_HEIGHT),
                )));
                self.pending_illustration = description.illustration_name;
                self.illustration = None;
                self.illustration_bg = None;
                return;
            }
        }
        let description = server
            .client
            .describe("/_describe/character/create", None, None)
            .unwrap();
        self.setup_no_home_image();
        self.engine = Some(Box::new(DescriptionEngine::new(
            None,
            // FIXME: manage error cases
            description.clone(),
            server.client.clone(),
            None,
            true,
            TileSheet::new(self.tile_sheet_image.clone(), TILE_WIDTH, TILE_HEIGHT),
        )));
        self.pending_illustration = description.illustration_name;
        self.illustration = None;
        self.illustration_bg = None;
    }

    fn setup_create_account(&mut self, address: server::ServerAddress) {
        println!("setup_create_account");
        let client = server::client::Client::new(address, ("".to_string(), "".to_string()));
        let description = client.describe("/account/create", None, None).unwrap();
        self.setup_home_image_background();
        self.engine = Some(Box::new(DescriptionEngine::new(
            None,
            description.clone(),
            client.clone(),
            None,
            true,
            TileSheet::new(self.tile_sheet_image.clone(), TILE_WIDTH, TILE_HEIGHT),
        )));
        self.pending_illustration = description.illustration_name;
        self.illustration = None;
        self.illustration_bg = None;
    }

    fn setup_create_character(&mut self) {
        println!("setup_create_character");
        let server = self.server.clone();
        let description = server
            .client
            .describe("/_describe/character/create", None, None)
            .unwrap();
        self.setup_home_image_background();
        self.engine = Some(Box::new(DescriptionEngine::new(
            None,
            description.clone(),
            server.client.clone(),
            None,
            true,
            TileSheet::new(self.tile_sheet_image.clone(), TILE_WIDTH, TILE_HEIGHT),
        )));
        self.pending_illustration = description.illustration_name;
        self.illustration = None;
        self.illustration_bg = None;
    }

    fn create_player(&self) -> Result<Option<Player>, Box<dyn Error>> {
        println!("create_player");
        // Server must exist at this step
        let server = self.server.clone();

        println!("Try to create Player with local data?");
        if let Some(character_id) = &server.character_id {
            println!("Character '{}' locally found", character_id);
            return match server.client.get_player(character_id) {
                Ok(player) => {
                    println!("Player found on server");
                    Ok(Some(player))
                }
                Err(server::client::ClientError::PlayerNotFound { message: _ }) => {
                    println!("Player NOT found on server");
                    Ok(None)
                }
                Err(client_error) => Err(Box::new(client_error)),
            };
        }

        println!("No local player found");
        return Ok(None);
    }

    fn create_upgrade_engine(&mut self, version: (u8, u8, u8), mandatory: bool) -> Box<dyn Engine> {
        println!("setup_upgrade_engine");
        Box::new(UpgradeEngine::new(
            version,
            mandatory,
            self.server.address.clone(),
        ))
    }

    fn setup_home_image_background(&mut self) {
        self.pending_home_image = match self.conf.get_from(Some("design"), "home_image_background")
        {
            None => None,
            Some(home_image_background) => Some(String::from(home_image_background)),
        }
    }

    fn setup_home_image(&mut self) {
        self.pending_home_image = match self.conf.get_from(Some("design"), "home_image") {
            None => None,
            Some(home_image_background) => Some(String::from(home_image_background)),
        }
    }

    fn setup_no_home_image(&mut self) {
        self.pending_home_image = None;
        self.home_image = None;
    }

    fn create_startup_engine(&mut self, disable_version_check: bool) -> Box<dyn Engine> {
        println!("create startup engine");

        if !disable_version_check {
            let server_version = self.server.client.get_version().unwrap();
            let client_version = util::str_version_to_tuple(VERSION);
            println!("Check is compatible");
            if !util::is_compatible_versions(server_version, client_version) {
                println!("Version is not compatible");
                let last_compatible_version = util::get_last_compatible_version(server_version);
                self.setup_home_image_background();
                return self.create_upgrade_engine(last_compatible_version, true);
            } else {
                println!("Version is compatible");
                let last_compatible_version = util::get_last_compatible_version(server_version);
                println!(
                    "Is there newer version ? ({:?} != {:?})",
                    last_compatible_version, client_version
                );
                if last_compatible_version != client_version {
                    self.setup_home_image_background();
                    return self.create_upgrade_engine(last_compatible_version, false);
                }
            }
        }

        Box::new(LoginEngine::new(
            self.server.address.clone(),
            None,
            self.get_server_last_username(self.server.address.clone()),
        ))
    }

    fn setup_zone_engine(&mut self, request_clicks: Option<RequestClicks>) {
        println!("setup_zone_engine");
        // Player and Server must exist at this step
        let server = self.server.clone();
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
        let world_tile_type_id = self.server.world.rows[player.world_position.0 as usize].cols
            [player.world_position.1 as usize]
            .clone();
        let level = Level::new(&zone_raw, &tiles, world_tile_type_id).unwrap();

        let mut socket = ZoneSocket::new(format!(
            "{}/ws/zones/{}/{}/events?character_id={}",
            server
                .address
                .with_credentials(&server.client.credentials.0, &server.client.credentials.1,),
            player.world_position.0,
            player.world_position.1,
            player.id
        ));
        if server.address.secure {
            socket.connect();
        } else {
            socket.connect_unsecure();
        };

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
            characters.insert(character.id.clone(), character);
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
        let mut builds: HashMap<i32, Build> = HashMap::new();
        for build in builds_list.into_iter() {
            builds.insert(build.id, build);
        }

        // ANIMATED CORPSES
        let animated_corpses_list = server
            .client
            .get_animated_corpses(player.world_position.0, player.world_position.1)
            .unwrap();
        let mut animated_corpses: HashMap<i32, AnimatedCorpse> = HashMap::new();
        for animated_corpse in animated_corpses_list.into_iter() {
            animated_corpses.insert(animated_corpse.id, animated_corpse);
        }

        let mut avatars: Vec<String> = vec![];
        for (_, character) in &characters {
            let avatar_uuid = if character.avatar_is_validated {
                if let Some(avatar_uuid) = &character.avatar_uuid {
                    avatar_uuid.to_string()
                } else {
                    "0000".to_string()
                }
            } else {
                "0000".to_string()
            };
            match server.client.cache_media(&format!(
                "character_avatar__zone_thumb__{}.png",
                avatar_uuid
            )) {
                Ok(_) => {
                    if !avatars.contains(&avatar_uuid) {
                        avatars.push(avatar_uuid)
                    }
                }
                Err(error) => {
                    eprintln!("Error when get avatar {} : {}", avatar_uuid, error)
                }
            };
        }

        self.engine = Some(Box::new(ZoneEngine::new(
            tiles,
            tile_sheet_image,
            avatars,
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
            animated_corpses,
            request_clicks,
        )));
        self.setup_no_home_image();
    }

    fn proceed_main_message(&mut self, main_message: MainMessage) {
        self.pending_action = Some(main_message);
    }
}

impl Game for MyGame {
    type Input = MyGameInput;
    type LoadingScreen = loading_screen::ProgressBar;
    const TICKS_PER_SECOND: u16 = 30;

    fn load(_window: &Window) -> Task<MyGame> {
        // Check if database exist in parent folder: This permit to use original client db when
        // using specified version of rollgui
        let db_in_parent_folder_path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("client.db");
        let db = if db_in_parent_folder_path.is_file() {
            get_db(db_in_parent_folder_path.to_str().unwrap())
        } else {
            get_db("client.db")
        };

        let opt = args::Opt::from_args();
        let conf = get_conf(&opt.config_file_path);
        let pending_home_image: Option<String> = match conf.get_from(Some("design"), "home_image") {
            None => None,
            Some(file_path) => Some(String::from(file_path)),
        };
        let server_hostname = conf.get_from(Some("server"), "server_hostname").unwrap();
        let server_port = conf
            .get_from(Some("server"), "server_port")
            .unwrap()
            .parse::<u16>()
            .unwrap();
        let server_unsecure = match conf.get_from(Some("server"), "unsecure").unwrap_or("false") {
            "true" | "True" | "1" => true,
            _ => false,
        };
        let server_address = if server_unsecure {
            server::ServerAddress::unsecure(server_hostname, server_port)
        } else {
            server::ServerAddress::new(server_hostname, server_port)
        };
        let server = match server::Server::new(
            server::client::Client::new(server_address.clone(), ("".to_string(), "".to_string())),
            server_address,
            None,
        ) {
            Ok(server) => server,
            Err(err) => {
                eprintln!("Connexion error : {}", err);
                dialog::Message::new("Erreur de connexion")
                    .title("Erreur")
                    .show()
                    .expect("Could not display dialog box");
                exit(1)
            }
        };

        // Start background task of loading screen downloads
        let server_ = server.clone();
        thread::spawn(move || {
            match server_.client.get_loading_media_names() {
                Ok(loading_media_names) => {
                    for loading_media_name in loading_media_names {
                        server_.client.cache_media(&loading_media_name).unwrap();
                    }
                }
                Err(error) => {
                    eprintln!("Error when get loading media names : {}", error);
                }
            };
        });

        graphics::Image::load("resources/graphics.png").map(|image| MyGame {
            conf,
            engine: None,
            tile_sheet_image: image,
            db,
            server,
            player: None,
            exit_requested: false,
            pending_action: None,
            loading_displayed: false,
            last_tick: SystemTime::now(),
            pending_illustration: None,
            illustration: None,
            illustration_bg: None,
            pending_home_image,
            home_image: None,
            loading_image_to_set: false,
            loading_image: None,
        })
    }

    fn interact(&mut self, input: &mut MyGameInput, window: &mut Window) {
        if self.engine.is_none() {
            return;
        }

        if let Some(pending_illustration) = self.pending_illustration.clone() {
            self.pending_illustration = None;
            self.illustration = None;
            self.illustration_bg = None;

            match graphics::Image::new(window.gpu(), format!("cache/{}", &pending_illustration)) {
                Ok(image) => self.illustration = Some(image),
                Err(error) => {
                    eprintln!(
                        "Error when loading illustration {}: {}",
                        pending_illustration, error
                    )
                }
            };

            match graphics::Image::new(window.gpu(), format!("cache/bg/{}", &pending_illustration))
            {
                Ok(image) => self.illustration_bg = Some(image),
                Err(error) => {
                    eprintln!(
                        "Error when loading illustration bg {}: {}",
                        pending_illustration, error
                    )
                }
            };
        }

        if let Some(pending_home_image) = self.pending_home_image.clone() {
            self.pending_home_image = None;
            self.home_image = None;

            match graphics::Image::new(window.gpu(), &pending_home_image) {
                Ok(image) => self.home_image = Some(image),
                Err(error) => {
                    eprintln!(
                        "Error when loading home image {}: {}",
                        pending_home_image, error
                    )
                }
            };
        }

        // Search for loading screen
        if self.loading_image_to_set {
            self.loading_image_to_set = false;
            let mut loadings: Vec<String> = vec![];
            for path in glob("cache/loading__*.png").expect("Failed to read glob pattern") {
                match path {
                    Ok(path) => match path.to_str() {
                        Some(path_) => loadings.push(path_.to_string()),
                        None => {}
                    },
                    Err(e) => {
                        eprintln!("Failed to read path: {}", e);
                    }
                }
            }
            if let Some(loading_path) = loadings.choose(&mut rand::thread_rng()) {
                match graphics::Image::new(window.gpu(), loading_path) {
                    Ok(image) => self.loading_image = Some(image),
                    Err(error) => {
                        eprintln!(
                            "Error when loading loading image {}: {}",
                            loading_path, error
                        )
                    }
                };
            }
        }

        match self.engine.as_mut().unwrap().interact(input, window) {
            Some(main_message) => self.proceed_main_message(main_message),
            None => {}
        }
    }

    fn update(&mut self, window: &Window) {
        if self.engine.is_none() {
            self.setup_home_image();
            self.engine = Some(self.create_startup_engine(false));
        }

        if self.loading_displayed {
            let main_message = self.pending_action.as_ref().unwrap().clone();
            self.pending_action = None;
            self.loading_displayed = false;

            match main_message {
                MainMessage::StartupToZone {
                    disable_version_check,
                } => {
                    println!("Set startup engine");
                    self.setup_home_image();
                    self.engine = Some(self.create_startup_engine(disable_version_check));
                    self.loading_image_to_set = true;
                }
                MainMessage::ToDescriptionWithDescription {
                    description,
                    back_url,
                    client,
                } => {
                    let player = match self.player.as_ref() {
                        None => None,
                        Some(player) => Some(player.clone()),
                    };
                    self.setup_no_home_image();
                    self.engine = Some(Box::new(DescriptionEngine::new(
                        player,
                        description.clone(),
                        client,
                        back_url.clone(),
                        false,
                        TileSheet::new(self.tile_sheet_image.clone(), TILE_WIDTH, TILE_HEIGHT),
                    )));
                    self.pending_illustration = description.illustration_name;
                    self.illustration = None;
                    self.illustration_bg = None;
                    self.loading_image_to_set = true;
                }
                MainMessage::CreateAccount { address } => {
                    self.setup_create_account(address);
                }
                MainMessage::AccountCreated => {
                    self.setup_home_image();
                    self.engine = Some(self.create_startup_engine(true));
                }
                MainMessage::NewCharacterId { character_id } => {
                    self.server.character_id = Some(character_id.clone());
                    self.setup_startup_to_zone_engine(None);
                }
                MainMessage::EnterServer {
                    credentials,
                    character_id,
                } => {
                    println!("Enter server");
                    self.server.client.credentials = credentials;
                    self.server.character_id = character_id;
                    self.set_server_last_username();

                    if self.server.character_id.is_none() {
                        self.setup_create_character();
                    } else {
                        self.setup_startup_to_zone_engine(None);
                    }
                }
                MainMessage::ToDescriptionWithUrl { url, back_url } => {
                    // FIXME: manage errors
                    let description = self.server.client.describe(&url, None, None).unwrap();
                    let player = if let Some(player) = self.player.as_ref() {
                        Some(player.clone())
                    } else {
                        None
                    };
                    self.setup_no_home_image();
                    self.engine = Some(Box::new(DescriptionEngine::new(
                        player,
                        description.clone(),
                        self.server.client.clone(),
                        back_url.clone(),
                        false,
                        TileSheet::new(self.tile_sheet_image.clone(), TILE_WIDTH, TILE_HEIGHT),
                    )));
                    self.pending_illustration = description.illustration_name;
                    self.illustration = None;
                    self.illustration_bg = None;
                    self.loading_image_to_set = true;
                }
                MainMessage::DescriptionToZone { request_clicks } => {
                    self.setup_startup_to_zone_engine(request_clicks);
                    self.loading_image_to_set = true;
                }
                MainMessage::ToStartup => {
                    self.setup_home_image();
                    self.engine = Some(self.create_startup_engine(true));
                }
                MainMessage::ToExit => {
                    self.setup_home_image_background();
                    self.engine = Some(Box::new(ExitEngine::new()));
                }
                MainMessage::ExitRequested => self.exit_requested = true,
                MainMessage::ToWorld => {
                    self.setup_no_home_image();
                    self.engine = Some(Box::new(WorldEngine::new(
                        self.server.clone(),
                        self.tile_sheet_image.clone(),
                        self.player.as_ref().unwrap().clone(),
                    )));
                    self.loading_image_to_set = true;
                }
            }
        }

        match self.engine.as_mut().unwrap().update(window) {
            Some(main_message) => self.proceed_main_message(main_message),
            None => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        if self.engine.is_none() {
            return;
        }

        // Slow down computing to preserve cpu
        util::sleep_if_required(TARGET_FRAME_DURATION_MS, &self.last_tick);
        self.last_tick = SystemTime::now();

        if self.pending_action.is_some() {
            frame.clear(Color::BLACK);
        } else {
            if self.illustration_bg.is_some() {
                self.engine
                    .as_mut()
                    .unwrap()
                    .draw(frame, timer, self.illustration_bg.clone())
            } else if self.home_image.is_some() {
                self.engine
                    .as_mut()
                    .unwrap()
                    .draw(frame, timer, self.home_image.clone())
            } else {
                self.engine.as_mut().unwrap().draw(frame, timer, None)
            }
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
        if self.engine.is_none() {
            return;
        }

        match self.engine.as_mut().unwrap().react(event, window) {
            Some(main_message) => self.proceed_main_message(main_message),
            None => {}
        }
    }

    fn layout(&mut self, window: &Window) -> Element {
        if self.engine.is_none()
            || self.pending_action.is_some()
            || self.pending_illustration.is_some()
        {
            if self.pending_action.is_some() {
                self.loading_displayed = true;
            }
            let mut column = Column::new()
                .width(window.width() as u32)
                .height(window.height() as u32)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .spacing(20);

            if let Some(loading_image) = &self.loading_image {
                column =
                    column.push(Image::new(loading_image).height((window.height() * 0.8) as u32));
            }

            column = column.push(
                Text::new("Chargement ...")
                    .size(50)
                    .height(60)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            );

            column.into()
        } else {
            self.engine
                .as_mut()
                .unwrap()
                .layout(window, self.illustration.clone())
        }
    }
}
