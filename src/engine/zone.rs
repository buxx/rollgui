use crate::engine::Engine;
use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::player::Player;
use crate::entity::resource::Resource;
use crate::entity::stuff::Stuff;
use crate::event::ZoneEventType;
use crate::game::{TILE_HEIGHT, TILE_WIDTH};
use crate::input::MyGameInput;
use crate::level::Level;
use crate::message::{MainMessage, Message};
use crate::server::Server;
use crate::sheet::TileSheet;
use crate::socket::ZoneSocket;
use crate::tile::zone::Tiles;
use crate::{event, util};
use coffee::graphics::{Batch, Color, Frame, Sprite, Window};
use coffee::input::keyboard;
use coffee::ui::widget::state_less_button;
use coffee::ui::widget::state_less_button::StateLessButton;
use coffee::ui::{button, Align, Button, Column, Element, Row, Text};
use coffee::{graphics, Timer};
use std::collections::HashMap;
use std::time::Instant;

const START_SCREEN_X: i16 = 400;
const START_SCREEN_Y: i16 = 0;

pub struct ZoneEngine {
    tiles: Tiles,
    tile_sheet: TileSheet,
    tile_sheet_batch: Batch,
    start_screen_x: i16,
    start_screen_y: i16,
    end_screen_x: i16,
    end_screen_y: i16,
    start_zone_row_i: i16,
    start_zone_col_i: i16,
    server: Server,
    player: Player,
    level: Level,
    socket: ZoneSocket,
    card_menu_button_state: button::State,
    events_menu_button_state: button::State,
    business_menu_button_state: button::State,
    affinities_menu_button_state: button::State,
    zone_menu_button_state: button::State,
    zone_messages_menu_button_state: button::State,
    conversations_menu_button_state: button::State,
    inventory_menu_button_state: button::State,
    action_menu_button_state: button::State,
    build_menu_button_state: button::State,
    exit_menu_button_state: button::State,
    resume_text: Vec<(String, Option<String>)>,
    around_text: Vec<(String, Option<String>)>,
    around_wait: Option<Instant>,
    menu_blinker: util::Blinker<char>,
    characters: HashMap<String, Character>,
    stuffs: HashMap<String, Stuff>,
    resources: Vec<Resource>,
    builds: HashMap<String, Build>,
    builds_positions: Vec<(i16, i16)>,
    link_button_ids: HashMap<String, i32>,
    link_button_pressed: i32,
    link_button_urls: HashMap<i32, String>,
}

impl ZoneEngine {
    pub fn new(
        tiles: Tiles,
        tile_sheet_image: graphics::Image,
        tile_width: i16,
        tile_height: i16,
        player: Player,
        server: Server,
        level: Level,
        socket: ZoneSocket,
        resume_text: Vec<(String, Option<String>)>,
        characters: HashMap<String, Character>,
        stuffs: HashMap<String, Stuff>,
        resources: Vec<Resource>,
        builds: HashMap<String, Build>,
    ) -> Self {
        let mut builds_positions = vec![];
        for (_, build) in builds.iter() {
            builds_positions.push((build.row_i as i16, build.col_i as i16));
        }

        let mut zone_engine = Self {
            tiles,
            tile_sheet: TileSheet::new(tile_sheet_image.clone(), tile_width, tile_height),
            tile_sheet_batch: Batch::new(tile_sheet_image.clone()),
            start_screen_x: START_SCREEN_X,
            start_screen_y: START_SCREEN_Y,
            end_screen_x: 0,
            end_screen_y: 0,
            start_zone_row_i: 0,
            start_zone_col_i: 0,
            server: server,
            player,
            level,
            socket,
            card_menu_button_state: button::State::new(),
            events_menu_button_state: button::State::new(),
            business_menu_button_state: button::State::new(),
            affinities_menu_button_state: button::State::new(),
            zone_menu_button_state: button::State::new(),
            zone_messages_menu_button_state: button::State::new(),
            conversations_menu_button_state: button::State::new(),
            inventory_menu_button_state: button::State::new(),
            action_menu_button_state: button::State::new(),
            build_menu_button_state: button::State::new(),
            exit_menu_button_state: button::State::new(),
            resume_text,
            around_text: vec![],
            around_wait: None,
            menu_blinker: util::Blinker {
                items: HashMap::new(),
            },
            characters,
            stuffs,
            resources,
            builds,
            builds_positions,
            link_button_ids: HashMap::new(),
            link_button_pressed: -1,
            link_button_urls: HashMap::new(),
        };
        zone_engine.update_link_button_data();
        zone_engine
    }

    fn update_link_button_data(&mut self) {
        let mut link_button_counter: i32 = 0;
        self.link_button_ids = HashMap::new();
        self.link_button_urls = HashMap::new();

        for (text, url) in self.resume_text.iter() {
            if let Some(url_) = url {
                self.link_button_ids
                    .insert(text.clone(), link_button_counter);
                self.link_button_urls
                    .insert(link_button_counter, url_.clone());

                link_button_counter += 1;
            }
        }

        for (text, url) in self.around_text.iter() {
            if let Some(url_) = url {
                self.link_button_ids
                    .insert(text.clone(), link_button_counter);
                self.link_button_urls
                    .insert(link_button_counter, url_.clone());

                link_button_counter += 1;
            }
        }
    }

    fn get_zone_sprites(&mut self, replace_by_back: Option<String>) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];
        let can_display_rows =
            ((self.end_screen_y - self.start_screen_y) / self.tile_sheet.get_tile_height()) + 0;
        let can_display_cols =
            ((self.end_screen_x - self.start_screen_x) / self.tile_sheet.get_tile_width()) + 0;

        for absolute_row_i in 0..can_display_rows {
            let zone_row_i = absolute_row_i + self.start_zone_row_i;
            if zone_row_i >= self.level.rows.len() as i16 || zone_row_i < 0 {
                continue;
            }
            let row = &self.level.rows[zone_row_i as usize];

            for absolute_col_i in 0..can_display_cols {
                let zone_col_i = absolute_col_i + self.start_zone_col_i;
                if zone_col_i >= row.cols.len() as i16 || zone_col_i < 0 {
                    continue;
                }

                // If build is here, do not draw tile
                if replace_by_back.is_none()
                    && self.builds_positions.contains(&(zone_row_i, zone_col_i))
                {
                    continue;
                }

                let mut tile_type_id = row.cols[zone_col_i as usize].clone();
                if tile_type_id != "NOTHING" {
                    if let Some(replace_by_back_) = &replace_by_back {
                        tile_type_id = format!("BACK_{}", replace_by_back_).clone();
                    };
                };

                sprites.push(self.tile_sheet.create_sprite_for(
                    &tile_type_id,
                    (self.tile_sheet.get_tile_width() * absolute_col_i) + self.start_screen_x,
                    (self.tile_sheet.get_tile_height() * absolute_row_i) + self.start_screen_y,
                ));
            }
        }

        sprites
    }

    fn get_real_x(&self, x: i16) -> i16 {
        x + START_SCREEN_X - (self.start_zone_col_i * TILE_WIDTH)
    }

    fn get_real_y(&self, y: i16) -> i16 {
        y + START_SCREEN_Y - (self.start_zone_row_i * TILE_HEIGHT)
    }

    fn get_characters_sprites(&mut self) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];

        sprites.push(self.tile_sheet.create_sprite_for(
            "CHARACTER",
            self.get_real_x(self.player.x),
            self.get_real_y(self.player.y),
        ));

        for character in self.characters.values().into_iter() {
            let real_x = self.get_real_x(character.position().1 as i16 * TILE_WIDTH);
            let real_y = self.get_real_y(character.position().0 as i16 * TILE_HEIGHT);
            if real_x < 0
                || real_x < START_SCREEN_X
                || real_x > self.end_screen_x
                || real_y < 0
                || real_y > self.end_screen_y
            {
                continue;
            }

            sprites.push(
                self.tile_sheet
                    .create_sprite_for("CHARACTER", real_x, real_y),
            );
        }

        sprites
    }

    fn get_stuff_sprites(&mut self) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];

        for stuff in self.stuffs.values().into_iter() {
            let real_x = self.get_real_x(stuff.position().1 as i16 * TILE_WIDTH);
            let real_y = self.get_real_y(stuff.position().0 as i16 * TILE_HEIGHT);
            if real_x < 0
                || real_x < START_SCREEN_X
                || real_x > self.end_screen_x
                || real_y < 0
                || real_y > self.end_screen_y
            {
                continue;
            }

            for class in stuff.get_classes().iter().rev() {
                if self.tile_sheet.have_id(class) {
                    sprites.push(self.tile_sheet.create_sprite_for(class, real_x, real_y));
                    break;
                }
            }
        }

        sprites
    }

    fn get_resource_sprites(&mut self) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];

        for resource in self.resources.iter() {
            let real_x = self.get_real_x(resource.position().1 as i16 * TILE_WIDTH);
            let real_y = self.get_real_y(resource.position().0 as i16 * TILE_HEIGHT);
            if real_x < 0
                || real_x < START_SCREEN_X
                || real_x > self.end_screen_x
                || real_y < 0
                || real_y > self.end_screen_y
            {
                continue;
            }

            sprites.push(
                self.tile_sheet
                    .create_sprite_for("RESOURCE_GENERIC", real_x, real_y),
            );
        }

        sprites
    }

    fn get_build_sprites(&mut self) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];

        for build in self.builds.values().into_iter() {
            let real_x = self.get_real_x(build.position().1 as i16 * TILE_WIDTH);
            let real_y = self.get_real_y(build.position().0 as i16 * TILE_HEIGHT);
            if real_x < 0
                || real_x < START_SCREEN_X
                || real_x > self.end_screen_x
                || real_y < 0
                || real_y > self.end_screen_y
            {
                continue;
            }

            for class in build.get_classes().iter().rev() {
                if self.tile_sheet.have_id(class) {
                    sprites.push(self.tile_sheet.create_sprite_for(class, real_x, real_y));
                    break;
                }
            }
        }

        sprites
    }

    fn try_travel(&mut self, moves: &Vec<(i16, i16)>) -> Option<MainMessage> {
        for try_player_move in moves.iter() {
            // Test if next requested move is an travel
            if let Some(next_position) = self
                .player
                .next_position(try_player_move.0, try_player_move.1)
            {
                // TODO: current check is tile will be None before get corner because get_corner algorithm
                // is ... not very working well
                let next_tile_id = self.level.tile_id(next_position.0, next_position.1);
                if "NOTHING" == next_tile_id {
                    if let Some(corner) = util::get_corner(
                        self.level.width as i16,
                        self.level.height as i16,
                        next_position.0,
                        next_position.1,
                    ) {
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
                            return Some(MainMessage::ToDescriptionWithUrl {
                                url,
                                back_url: None,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    fn try_player_moves(&mut self, moves: &Vec<(i16, i16)>) -> (bool, bool) {
        let mut player_have_move = (false, false);

        for try_player_move in moves.iter() {
            let player_have_this_move = self.try_player_move(*try_player_move);
            if player_have_this_move.0 {
                player_have_move.0 = player_have_this_move.0
            }
            if player_have_this_move.1 {
                player_have_move.1 = player_have_this_move.1
            }
        }

        player_have_move
    }

    fn try_player_move(&mut self, move_: (i16, i16)) -> (bool, bool) {
        let try_next_position = (self.player.x + move_.0, self.player.y + move_.1);
        let try_next_tile = util::get_tile_position_for_xy(
            self.tile_sheet.get_tile_width(),
            self.tile_sheet.get_tile_height(),
            try_next_position.0,
            try_next_position.1,
        );
        let next_tile_id = self.level.tile_id(try_next_tile.0, try_next_tile.1);
        if self.tiles.browseable(&next_tile_id) {
            return self.player.try_move_by(move_.0, move_.1);
        }
        (false, false)
    }

    fn update_zone_display(&mut self) {
        let player_from_top_left_x = self.get_real_x(self.player.x) - self.start_screen_x;
        let player_from_top_left_y = self.get_real_y(self.player.y) - self.start_screen_y;
        let player_bottom_right_left_x = self.end_screen_x - self.get_real_x(self.player.x);
        let player_from_bottom_right_y = self.end_screen_y - self.get_real_y(self.player.y);
        let player_from_top_left_cols = player_from_top_left_x / TILE_WIDTH;
        let player_from_top_left_rows = player_from_top_left_y / TILE_HEIGHT;
        let player_from_bottom_right_cols = player_bottom_right_left_x / TILE_WIDTH;
        let player_from_bottom_right_rows = player_from_bottom_right_y / TILE_HEIGHT;

        if player_from_top_left_cols < 5 {
            self.start_zone_col_i -= 1;
        }
        if player_from_top_left_rows < 5 {
            self.start_zone_row_i -= 1;
        }
        if player_from_bottom_right_cols < 5 {
            self.start_zone_col_i += 1;
        }
        if player_from_bottom_right_rows < 5 {
            self.start_zone_row_i += 1;
        }
    }
}

impl Drop for ZoneEngine {
    fn drop(&mut self) {
        // FIXME BS NOW: close websocket here
    }
}

impl Engine for ZoneEngine {
    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        frame.clear(Color::BLACK);

        if timer.has_ticked() {
            let mut sprites: Vec<Sprite> = vec![];

            sprites.extend(self.get_zone_sprites(Some(self.level.world_tile_type_id.clone())));
            sprites.extend(self.get_zone_sprites(None));
            sprites.extend(self.get_build_sprites());
            sprites.extend(self.get_stuff_sprites());
            sprites.extend(self.get_resource_sprites());
            sprites.extend(self.get_characters_sprites());

            self.tile_sheet_batch.clear();
            self.tile_sheet_batch.extend(sprites);
            self.tile_sheet_batch.draw(&mut frame.as_target());
        }
    }

    fn update(&mut self, window: &Window) -> Option<MainMessage> {
        // FIXME BS NOW: changer ici start_zone_row_i, start_zone_col_i en fonction des changements (deplacement perso, etc)
        self.end_screen_x = window.width() as i16;
        self.end_screen_y = window.height() as i16;
        self.update_zone_display();

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
                    self.update_link_button_data();
                }
                _ => println!("unknown event type {:?}", &event.event_type),
            }
        }

        None
    }

    fn interact(&mut self, input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        let mut try_player_moves: Vec<(i16, i16)> = vec![];

        if !input.keys_pressed.is_empty() {
            if input.keys_pressed.contains(&keyboard::KeyCode::Right) {
                try_player_moves.push((1, 0));
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Left) {
                try_player_moves.push((-1, 0));
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Up) {
                try_player_moves.push((0, -1));
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Down) {
                try_player_moves.push((0, 1));
            }
        }

        match input.key_code {
            Some(keyboard::KeyCode::Escape) => {
                input.key_code = None;
                return Some(MainMessage::ToExit);
            }
            _ => {}
        }

        if let Some(main_message) = self.try_travel(&try_player_moves) {
            return Some(main_message);
        }

        let mut player_have_move = (false, false);
        if try_player_moves.len() != 0 {
            player_have_move = self.try_player_moves(&try_player_moves);
            if !player_have_move.0 {
                // try diags is any
                let mut additional_try_player_mode: (i16, i16) = (0, 0);
                if try_player_moves.len() > 1 {
                    for try_player_move in try_player_moves.iter() {
                        additional_try_player_mode.0 += try_player_move.0;
                        additional_try_player_mode.1 += try_player_move.1;
                    }
                }
                player_have_move = self.try_player_move(additional_try_player_mode);
            }
        }
        if player_have_move.1 {
            // NOTE: There is problem with moves and we send ws move on NOTHING tile :/ skip it
            let next_tile_id = self
                .level
                .tile_id(self.player.position.0 as i16, self.player.position.1 as i16);
            if self.tiles.browseable(&next_tile_id) {
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
            }
        } else {
            if let Some(around_wait) = self.around_wait.as_ref() {
                if around_wait.elapsed().as_millis() > 350 {
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
        None
    }

    fn react(&mut self, event: Message, _window: &mut Window) -> Option<MainMessage> {
        match event {
            Message::CardMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/_describe/character/{}/card", self.player.id).to_string(),
                    back_url: None,
                })
            }
            Message::EventsMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/_describe/character/{}/events", self.player.id).to_string(),
                    back_url: None,
                })
            }
            Message::BusinessMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/business/{}", self.player.id).to_string(),
                    back_url: None,
                })
            }
            Message::AffinitiesMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/affinity/{}", self.player.id).to_string(),
                    back_url: None,
                })
            }
            Message::ZoneMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!(
                        "/zones/{}/{}/describe/{}",
                        self.player.world_position.0, self.player.world_position.1, self.player.id
                    )
                    .to_string(),
                    back_url: None,
                })
            }
            Message::ZoneMessagesMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!(
                        "/zones/{}/{}/messages?character_id={}",
                        self.player.world_position.0, self.player.world_position.1, self.player.id
                    )
                    .to_string(),
                    back_url: None,
                })
            }
            Message::ConversationsMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/conversation/{}", self.player.id).to_string(),
                    back_url: None,
                })
            }
            Message::InventoryMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/_describe/character/{}/inventory", self.player.id).to_string(),
                    back_url: None,
                })
            }
            Message::ActionMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/_describe/character/{}/on_place_actions", self.player.id)
                        .to_string(),
                    back_url: None,
                })
            }
            Message::BuildMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/_describe/character/{}/build_actions", self.player.id)
                        .to_string(),
                    back_url: None,
                })
            }
            Message::ExitMenuButtonPressed => return Some(MainMessage::ToExit),
            Message::LinkButtonPressed(id) => {
                self.link_button_pressed = id;
            }
            Message::LinkButtonReleased(id) => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: self.link_button_urls.get(&id).unwrap().clone(),
                    back_url: None,
                });
            }
            _ => {}
        }
        None
    }

    fn layout(&mut self, window: &Window) -> Element<Message> {
        let event_label = if self.player.unread_event && self.menu_blinker.visible(500, 'E') {
            "*Événements*"
        } else {
            "Événements"
        };
        let business_label =
            if self.player.unread_transactions && self.menu_blinker.visible(500, 'B') {
                "*Commerce*"
            } else {
                "Commerce"
            };
        let affinity_label =
            if self.player.unvote_affinity_relation && self.menu_blinker.visible(500, 'A') {
                "*Affinités*"
            } else {
                "Affinités"
            };
        let zone_message_label =
            if self.player.unread_zone_message && self.menu_blinker.visible(500, 'M') {
                "*Chat*"
            } else {
                "Chat"
            };
        let conversation_label =
            if self.player.unread_conversation && self.menu_blinker.visible(500, 'C') {
                "*Conversations*"
            } else {
                "Conversations"
            };

        let left_menu = Column::new()
            .width((START_SCREEN_X / 2) as u32)
            .height(window.height() as u32)
            // .align_items(Align::Center)
            // .justify_content(Justify::Center)
            .padding(0)
            .spacing(5)
            .push(
                Button::new(&mut self.card_menu_button_state, "Fiche")
                    .class(button::Class::Secondary)
                    .on_press(Message::CardMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.events_menu_button_state, event_label)
                    .class(button::Class::Secondary)
                    .on_press(Message::EventsMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.business_menu_button_state, business_label)
                    .class(button::Class::Secondary)
                    .on_press(Message::BusinessMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.affinities_menu_button_state, affinity_label)
                    .class(button::Class::Secondary)
                    .on_press(Message::AffinitiesMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.zone_menu_button_state, "Zone")
                    .class(button::Class::Secondary)
                    .on_press(Message::ZoneMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(
                    &mut self.zone_messages_menu_button_state,
                    zone_message_label,
                )
                .class(button::Class::Secondary)
                .on_press(Message::ZoneMessagesMenuButtonPressed)
                .width(175),
            )
            .push(
                Button::new(
                    &mut self.conversations_menu_button_state,
                    conversation_label,
                )
                .class(button::Class::Secondary)
                .on_press(Message::ConversationsMenuButtonPressed)
                .width(175),
            )
            .push(
                Button::new(&mut self.inventory_menu_button_state, "Inventaire")
                    .class(button::Class::Secondary)
                    .on_press(Message::InventoryMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.action_menu_button_state, "Actions")
                    .class(button::Class::Secondary)
                    .on_press(Message::ActionMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.build_menu_button_state, "Construire")
                    .class(button::Class::Secondary)
                    .on_press(Message::BuildMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.exit_menu_button_state, "Quitter")
                    .class(button::Class::Secondary)
                    .on_press(Message::ExitMenuButtonPressed)
                    .width(175),
            );

        let mut right_menu = Column::new()
            .width((START_SCREEN_X / 2) as u32)
            .height(window.height() as u32);
        for (text, url) in self.resume_text.iter() {
            if url.is_some() {
                let id = self.link_button_ids.get(text).unwrap().clone();
                right_menu = right_menu.push(
                    StateLessButton::new(
                        self.link_button_pressed == id,
                        &text,
                        Message::LinkButtonPressed(id),
                        Message::LinkButtonReleased(id),
                    )
                    .width(175)
                    .class(state_less_button::Class::Positive),
                );
            } else {
                right_menu = right_menu.push(Text::new(text));
            }
        }
        for (text, url) in self.around_text.iter() {
            if url.is_some() {
                let id = self.link_button_ids.get(text).unwrap().clone();
                right_menu = right_menu.push(
                    StateLessButton::new(
                        self.link_button_pressed == id,
                        &text,
                        Message::LinkButtonPressed(id),
                        Message::LinkButtonReleased(id),
                    )
                    .width(175)
                    .class(state_less_button::Class::Positive),
                );
            } else {
                right_menu = right_menu.push(Text::new(text));
            }
        }

        let layout = Row::new()
            .push(left_menu)
            .align_items(Align::Stretch)
            .push(right_menu);

        layout.into()
    }

    fn teardown(&mut self) {
        // TODO: manage case where fail to close
        self.socket.close().unwrap();
    }
}
