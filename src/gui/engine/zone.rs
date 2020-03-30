use doryen_rs::{DoryenApi, TextAlign};
use doryen_ui as ui;
use std::collections::HashMap;
use std::time::Instant;

use crate::color;
use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::player::Player;
use crate::entity::stuff::Stuff;
use crate::event;
use crate::event::ZoneEventType;
use crate::gui;
use crate::gui::action;
use crate::gui::engine::Engine;
use crate::server::Server;
use crate::tile::zone::Tiles;
use crate::util;
use crate::world::level::Level;
use crate::world::socket::ZoneSocket;

const UI_WIDTH: i32 = 20;
const UI_HEIGHT: i32 = 50;

pub struct ZoneEngine {
    server: Server,
    player: Player,
    characters: HashMap<String, Character>,
    stuffs: HashMap<String, Stuff>,
    builds: HashMap<String, Build>,
    socket: ZoneSocket,
    level: Level,
    tiles: Tiles,
    // Map coordinates where start to display it
    start_display_map_row_i: i32,
    start_display_map_col_i: i32,
    mouse_pos: (f32, f32),
    resume_text: Vec<(String, Option<String>)>,
    around_text: Vec<(String, Option<String>)>,
    around_wait: Option<Instant>,
    menu_blinker: util::Blinker<char>,
}

impl ZoneEngine {
    pub fn new(
        server: Server,
        player: Player,
        characters: HashMap<String, Character>,
        stuffs: HashMap<String, Stuff>,
        builds: HashMap<String, Build>,
        socket: ZoneSocket,
        level: Level,
        tiles: Tiles,
        start_display_map_row_i: i32,
        start_display_map_col_i: i32,
        resume_text: Vec<(String, Option<String>)>,
    ) -> Self {
        Self {
            server,
            player,
            characters,
            stuffs,
            builds,
            socket,
            level,
            tiles,
            start_display_map_row_i,
            start_display_map_col_i,
            mouse_pos: (0.0, 0.0),
            resume_text,
            around_text: vec![],
            around_wait: None,
            menu_blinker: util::Blinker {
                items: HashMap::new(),
            },
        }
    }

    pub fn can_move(&self, position: (i32, i32)) -> bool {
        let tile = self.level.tile_id(position);
        self.tiles.browseable(&tile)
    }
}

impl Engine for ZoneEngine {
    fn get_name(&self) -> &str {
        "ZONE"
    }

    fn update(
        &mut self,
        api: &mut dyn DoryenApi,
        width: i32,
        height: i32,
    ) -> Option<action::Action> {
        let _input = api.input();

        for event in self.socket.pending_events() {
            // TODO Move code ailleurs
            match event.event_type {
                ZoneEventType::PlayerMove {
                    to_row_i,
                    to_col_i,
                    character_id,
                } => {
                    if let Some(mut moved_character) =
                        self.characters.get_mut(character_id.as_str())
                    {
                        moved_character.zone_row_i = to_row_i;
                        moved_character.zone_col_i = to_col_i;
                    } else if character_id != self.player.id {
                        eprintln!("Unknown character {} for move", character_id)
                    }
                }
                ZoneEventType::CharacterEnter {
                    zone_row_i,
                    zone_col_i,
                    character_id,
                } => {
                    println!("{} is enter in zone", &character_id);
                    self.characters.insert(
                        character_id.clone(),
                        Character {
                            id: character_id.clone(),
                            zone_row_i,
                            zone_col_i,
                        },
                    );
                }
                ZoneEventType::CharacterExit { character_id } => {
                    if let None = self.characters.remove(&character_id) {
                        println!(
                            "{} left zone but was not in list of characters",
                            &character_id
                        );
                    } else {
                        println!("{} exit from zone", &character_id);
                    }
                }
                ZoneEventType::ThereIsAround { items } => {
                    self.around_text = items;
                }
                _ => println!("unknown event type {:?}", &event.event_type),
            }
        }

        let mut mov = self.player.move_from_input(api);
        let mut coef = 1.0 / std::f32::consts::SQRT_2;

        // Test if next requested move is an travel
        // FIXME: do only if there is a move
        let next = self.player.next_position((mov.0, mov.1));
        // TODO: current check is tile will be None before get corner because get_corner algorithm
        // is ... not very working well
        if "NOTHING" == self.level.tile_id((next.0, next.1)) {
            if let Some(corner) =
                util::get_corner(self.level.width, self.level.height, next.0, next.1)
            {
                let w_row_i = self.player.world_position.0;
                let w_col_i = self.player.world_position.1;
                let (to_row_i, to_col_i) = match corner {
                    util::CornerEnum::Top => (w_row_i - 1, w_col_i),
                    util::CornerEnum::TopRight => (w_row_i - 1, w_col_i + 1),
                    util::CornerEnum::Right => (w_row_i, w_col_i + 1),
                    util::CornerEnum::BottomRight => (w_row_i + 1, w_col_i + 1),
                    util::CornerEnum::Bottom => (w_row_i + 1, w_col_i),
                    util::CornerEnum::BottomLeft => (w_row_i + 1, w_col_i - 1),
                    util::CornerEnum::Left => (w_row_i, w_col_i - 1),
                    util::CornerEnum::TopLeft => (w_row_i - 1, w_col_i - 1),
                };

                // If world coordinates don't exist, do nothing
                if let Some(_) = self.server.world.tile_id(to_row_i, to_col_i) {
                    let url = format!(
                        "/_describe/character/{}/move-to-zone/{}/{}",
                        self.player.id, to_row_i, to_col_i
                    );
                    return Some(action::Action::ZoneToDescription { url });
                }
            }
        }

        if !self.can_move(self.player.next_position((mov.0, 0))) {
            mov.0 = 0;
            coef = 1.0;
        }
        if !self.can_move(self.player.next_position((0, mov.1))) {
            mov.1 = 0;
            coef = 1.0;
        }
        if self.player.move_by(mov, coef) {
            self.socket.send(event::ZoneEvent {
                event_type_name: String::from(event::PLAYER_MOVE),
                event_type: event::ZoneEventType::PlayerMove {
                    to_row_i: self.player.position.0 as i32,
                    to_col_i: self.player.position.1 as i32,
                    character_id: String::from(self.player.id.as_str()),
                },
            });
            self.around_wait = Some(Instant::now());
            self.around_text = vec![];
        } else {
            if let Some(around_wait) = self.around_wait.as_ref() {
                if around_wait.elapsed().as_millis() > 150 {
                    self.around_wait = None;
                    self.socket.send(event::ZoneEvent {
                        event_type_name: String::from(event::CLIENT_REQUIRE_AROUND),
                        event_type: event::ZoneEventType::ClientRequireAround {
                            zone_row_i: self.player.position.0 as i32,
                            zone_col_i: self.player.position.1 as i32,
                            character_id: String::from(self.player.id.as_str()),
                        },
                    });
                }
            }
        }

        self.start_display_map_row_i = self.player.position.0 as i32 - (height / 2);
        self.start_display_map_col_i = self.player.position.1 as i32 - (width / 2);

        self.mouse_pos = api.input().mouse_pos();
        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) {
        self.level.render(
            api,
            &self.tiles,
            self.start_display_map_row_i,
            self.start_display_map_col_i,
            width,
            height,
            0,
            UI_WIDTH / 2,
        );
        let con = api.con();

        // CHARACTERS
        for (_character_id, character) in self.characters.iter() {
            con.ascii(
                (character.zone_col_i - self.start_display_map_col_i) + (UI_WIDTH / 2),
                (character.zone_row_i - self.start_display_map_row_i) + 0,
                gui::CHAR_CHARACTER,
            );
            con.fore(
                (character.zone_col_i - self.start_display_map_col_i) + (UI_WIDTH / 2),
                (character.zone_row_i - self.start_display_map_row_i) + 0,
                color::WHITE,
            );
        }

        // STUFFS
        for (_stuff_id, stuff) in self.stuffs.iter() {
            con.ascii(
                (stuff.zone_col_i - self.start_display_map_col_i) + (UI_WIDTH / 2),
                (stuff.zone_row_i - self.start_display_map_row_i) + 0,
                gui::CHAR_TRUNK,
            );
            con.fore(
                (stuff.zone_col_i - self.start_display_map_col_i) + (UI_WIDTH / 2),
                (stuff.zone_row_i - self.start_display_map_row_i) + 0,
                color::WHITE,
            );
        }

        // BUILDS
        for (_build_id, build) in self.builds.iter() {
            con.ascii(
                (build.col_i - self.start_display_map_col_i) + (UI_WIDTH / 2),
                (build.row_i - self.start_display_map_row_i) + 0,
                gui::CHAR_GEARS,
            );
            con.fore(
                (build.col_i - self.start_display_map_col_i) + (UI_WIDTH / 2),
                (build.row_i - self.start_display_map_row_i) + 0,
                color::WHITE,
            );
        }
        self.player
            .render(api, width as i32, height as i32, 0, UI_WIDTH / 2);

        let fps = api.fps();
        api.con().print_color(
            1,
            20,
            &format!(
                "row {} / col {} {}fps",
                self.mouse_pos.1 as i32, self.mouse_pos.0 as i32, fps
            ),
            TextAlign::Left,
            None,
        );
        api.con().back(
            self.mouse_pos.0 as i32,
            self.mouse_pos.1 as i32,
            (255, 255, 255, 255),
        );
    }
    fn resize(&mut self, _api: &mut dyn DoryenApi) {}

    fn teardown(&mut self) {
        // TODO: manage case where fail to close
        self.socket.close().unwrap();
    }

    fn build_ui(
        &mut self,
        ctx: &mut ui::Context,
        _width: i32,
        _height: i32,
    ) -> Option<gui::action::Action> {
        ctx.window_begin("main_windows", 0, 0, UI_WIDTH, UI_HEIGHT);
        ctx.frame_begin("margin", "margin", UI_WIDTH, UI_HEIGHT);

        if ctx
            .button("worldmap_button", "Carte du monde")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToWorld);
        }

        if ctx
            .button("card_button", "Fiche")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!("/_describe/character/{}/card", self.player.id).to_string(),
            });
        }

        let events_label = if self.player.unread_event && self.menu_blinker.visible(500, 'E') {
            "*Evenements*"
        } else {
            "Evenements"
        };
        if ctx
            .button("events_button", events_label)
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!("/_describe/character/{}/events", self.player.id).to_string(),
            });
        }

        if ctx
            .button("affinity_button", "Affinités")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!(
                    "/affinity/{}",
                    self.player.id,
                )
                .to_string(),
            });
        }

        if ctx
            .button("zone_button", "Zone")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!(
                    "/zones/{}/{}/describe",
                    self.player.world_position.0, self.player.world_position.1,
                )
                .to_string(),
            });
        }

        let zone_message_label =
            if self.player.unread_zone_message && self.menu_blinker.visible(500, 'Z') {
                "*Messages de zone*"
            } else {
                "Messages de zone"
            };
        if ctx
            .button("zone_messages_button", zone_message_label)
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!(
                    "/zones/{}/{}/messages?character_id={}",
                    self.player.world_position.0, self.player.world_position.1, self.player.id
                )
                .to_string(),
            });
        }

        let conversation_label =
            if self.player.unread_conversation && self.menu_blinker.visible(500, 'C') {
                "*Conversations*"
            } else {
                "Conversations"
            };
        if ctx
            .button("conversation_button", conversation_label)
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!("/conversation/{}", self.player.id).to_string(),
            });
        }

        if ctx
            .button("inventory_button", "Inventaire")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!("/_describe/character/{}/inventory", self.player.id).to_string(),
            });
        }

        if ctx
            .button("action_button", "Actions")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!("/_describe/character/{}/on_place_actions", self.player.id)
                    .to_string(),
            });
        }

        if ctx
            .button("build_button", "Construire")
            .align(ui::TextAlign::Center)
            .pressed()
        {
            return Some(action::Action::ZoneToDescription {
                url: format!("/_describe/character/{}/build_actions", self.player.id).to_string(),
            });
        }

        ctx.label("");

        for (resume_text, optional_link) in self.resume_text.iter() {
            if let Some(link) = optional_link {
                if ctx
                    .button(resume_text, resume_text)
                    .align(ui::TextAlign::Left)
                    .pressed()
                {
                    return Some(action::Action::ZoneToDescription { url: link.clone() });
                }
            } else {
                ctx.label(resume_text);
            }
        }

        ctx.label("");
        if self.around_text.len() != 0 {
            ctx.label("Autour: ");
            for (around_text, optional_link) in self.around_text.iter() {
                if let Some(link) = optional_link {
                    if ctx
                        .button(around_text, around_text)
                        .align(ui::TextAlign::Left)
                        .pressed()
                    {
                        return Some(action::Action::ZoneToDescription { url: link.clone() });
                    }
                } else {
                    ctx.label(around_text);
                }
            }
            ctx.label("");
        }

        None
    }
}
