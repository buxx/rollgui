use crate::engine::Engine;
use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::corpse::AnimatedCorpse;
use crate::entity::player::Player;
use crate::entity::resource::Resource;
use crate::entity::stuff::Stuff;
use crate::event::{CharacterActionLink, TopBarMessageType, ZoneEventType};
use crate::game::{TILE_HEIGHT, TILE_WIDTH};
use crate::gui::lang::model::{Description, RequestClicks};
use crate::input::MyGameInput;
use crate::level::Level;
use crate::message::{self, MainMessage, Message};
use crate::server::client::{ClientError, ItemModel};
use crate::server::Server;
use crate::sheet::TileSheet;
use crate::socket::ZoneSocket;
use crate::tile::zone::Tiles;
use crate::ui::widget::fixed_button;
use crate::ui::widget::icon;
use crate::ui::widget::link::Link;
use crate::ui::widget::progress_bar;
use crate::ui::widget::sheet_button::SheetButton;
use crate::ui::widget::text;
use crate::ui::widget::text_input::TextInput;
use crate::ui::widget::thin_button;
use crate::ui::widget::thin_button::Button;
use crate::ui::Column;
use crate::ui::Element;
use crate::ui::Row;
use crate::{event, util};
use coffee::graphics::{
    Batch, Color, Frame, HorizontalAlignment, Image, Point, Rectangle, Sprite, VerticalAlignment,
    Window,
};
use coffee::input::keyboard;
use coffee::input::mouse;
use coffee::ui::Align;
use coffee::{graphics, Timer};
use crossbeam_channel::unbounded;
use pathfinding::prelude::{absdiff, astar};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const START_SCREEN_X: i16 = 0;
const LEFT_MENU_WIDTH: i16 = 200;
const RIGHT_MENU_WIDTH: i16 = 120;
const START_SCREEN_Y: i16 = 0;
const TEXT_INPUT_CHAT_ID: i32 = 0;
const CHAT_MESSAGE_COUNT: i32 = 15;
const TOP_BAR_BUTTON_WIDTH: u32 = 100;
const MARGIN_RIGHT_CHAT: u32 = 25;
const CHAT_LINE_HEIGHT: u32 = 20;
const BORDERS_TO_SEE_PLAYER_LEN: i16 = 15;
const QUICK_ACTION_ROW_HEIGHT: u32 = 50;

fn contains_string(classes: &Vec<String>, search: &str) -> bool {
    for class in classes.iter() {
        if class.eq(&search) {
            return true;
        }
    }

    false
}

#[derive(Debug, Clone)]
pub struct TopBar {
    text: String,
    text_color: Color,
    display_buttons: bool,
    on_click: Option<Message>,
}

pub struct Chat {
    messages: Vec<String>,
    conversation_id: Option<i32>,
}

pub struct ZoneEngine {
    i: i32,
    sprite_i: i32,
    tiles: Tiles,
    tile_sheet: TileSheet,
    tile_sheet_batch: Batch,
    avatars_to_load: Vec<String>,
    avatars: HashMap<String, (Batch, u16, u16)>,
    hover_character_id: Option<String>,
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
    world_menu_button_state: thin_button::State,
    card_menu_button_state: thin_button::State,
    events_menu_button_state: thin_button::State,
    business_menu_button_state: thin_button::State,
    affinities_menu_button_state: thin_button::State,
    zone_menu_button_state: thin_button::State,
    zone_messages_menu_button_state: thin_button::State,
    conversations_menu_button_state: thin_button::State,
    inventory_menu_button_state: thin_button::State,
    action_menu_button_state: thin_button::State,
    build_menu_button_state: thin_button::State,
    info_menu_button_state: thin_button::State,
    account_button_state: thin_button::State,
    exit_menu_button_state: thin_button::State,
    resume_text: Vec<ItemModel>,
    around_items_button_state: fixed_button::State,
    around_builds_button_state: fixed_button::State,
    around_characters_button_state: fixed_button::State,
    around_items_count: i32,
    around_builds_count: i32,
    around_characters_count: i32,
    around_wait: Option<Instant>,
    around_quick_actions: Vec<CharacterActionLink>,
    current_quick_action_link_pressed: Option<String>,
    blinker: util::Blinker<char>,
    characters: HashMap<String, Character>,
    stuffs: HashMap<String, Stuff>,
    resources: Vec<Resource>,
    builds: HashMap<i32, Build>,
    animated_corpses: HashMap<i32, AnimatedCorpse>,
    builds_positions: HashMap<(i16, i16), Vec<i32>>,
    link_button_ids: HashMap<String, i32>,
    link_button_pressed: i32,
    move_requested: Option<Vec<(i16, i16)>>,
    request_clicks: Option<RequestClicks>,
    cursor_position: Point,
    player_tile_id: String,
    player_move_ticker: util::Ticker,
    top_bar: Option<TopBar>,
    displaying_chat: bool,
    display_chat_required: bool,
    chat: Option<Chat>,
    chat_input_value: String,
    previous_chat_button_state: fixed_button::State,
    next_chat_button_state: fixed_button::State,
    unread_conversation_id: Vec<Option<i32>>,
    replace_top_bar_start: Option<SystemTime>,
    replace_top_bar_by: Option<TopBar>,
    send_quick_actions_transmitter: crossbeam_channel::Sender<String>,
    response_quick_actions_receiver: crossbeam_channel::Receiver<Result<Description, ClientError>>,
}

impl ZoneEngine {
    pub fn new(
        tiles: Tiles,
        tile_sheet_image: graphics::Image,
        avatars: Vec<String>,
        tile_width: i16,
        tile_height: i16,
        player: Player,
        server: Server,
        level: Level,
        socket: ZoneSocket,
        resume_text: Vec<ItemModel>,
        characters: HashMap<String, Character>,
        stuffs: HashMap<String, Stuff>,
        resources: Vec<Resource>,
        builds: HashMap<i32, Build>,
        animated_corpses: HashMap<i32, AnimatedCorpse>,
        request_clicks: Option<RequestClicks>,
    ) -> Self {
        let (top_bar, replace_top_bar_start) = if request_clicks.is_some() {
            (
                Some(TopBar {
                    text: "Appuyez sur Echap pour annuler le mode construction".to_string(),
                    text_color: Color::WHITE,
                    display_buttons: false,
                    on_click: Some(Message::DismissRequestClicks),
                }),
                None,
            )
        } else {
            if characters.len() > 0 {
                (
                    Some(TopBar {
                        text: "Appuyez sur ENTRER pour ouvrir le chat".to_string(),
                        text_color: Color::WHITE,
                        display_buttons: false,
                        on_click: Some(Message::RequestChat(None)),
                    }),
                    Some(SystemTime::now()),
                )
            } else {
                (None, None)
            }
        };

        let (send_quick_actions_transmitter, send_quick_actions_receiver) = unbounded::<String>();
        let (response_quick_actions_transmitter, response_quick_actions_receiver) = unbounded();
        let quick_action_server = server.clone();
        thread::spawn(move || loop {
            match send_quick_actions_receiver.recv() {
                Ok(link) => {
                    match response_quick_actions_transmitter
                        .send(quick_action_server.client.describe(&link, None, None))
                    {
                        Ok(_) => {}
                        Err(error) => {
                            eprintln!("Error when send quick action response : {}", error)
                        }
                    };
                }
                Err(_error) => {
                    // Commented to prevent multiple prints when closing
                    // eprintln!("Error when reading from quick actions channel : {}", error)
                }
            };
        });

        let mut zone_engine = Self {
            i: 0,
            sprite_i: 0,
            tiles,
            tile_sheet: TileSheet::new(tile_sheet_image.clone(), tile_width, tile_height),
            tile_sheet_batch: Batch::new(tile_sheet_image.clone()),
            avatars_to_load: avatars,
            avatars: HashMap::new(),
            hover_character_id: None,
            start_screen_x: START_SCREEN_X,
            start_screen_y: START_SCREEN_Y,
            end_screen_x: 0,
            end_screen_y: 0,
            start_zone_row_i: 0,
            start_zone_col_i: 0,
            server,
            player,
            level,
            socket,
            world_menu_button_state: thin_button::State::new(),
            card_menu_button_state: thin_button::State::new(),
            events_menu_button_state: thin_button::State::new(),
            business_menu_button_state: thin_button::State::new(),
            affinities_menu_button_state: thin_button::State::new(),
            zone_menu_button_state: thin_button::State::new(),
            zone_messages_menu_button_state: thin_button::State::new(),
            conversations_menu_button_state: thin_button::State::new(),
            inventory_menu_button_state: thin_button::State::new(),
            action_menu_button_state: thin_button::State::new(),
            build_menu_button_state: thin_button::State::new(),
            info_menu_button_state: thin_button::State::new(),
            account_button_state: thin_button::State::new(),
            exit_menu_button_state: thin_button::State::new(),
            resume_text,
            around_items_button_state: fixed_button::State::new(),
            around_builds_button_state: fixed_button::State::new(),
            around_characters_button_state: fixed_button::State::new(),
            around_items_count: 0,
            around_builds_count: 0,
            around_characters_count: 0,
            around_wait: None,
            around_quick_actions: vec![],
            current_quick_action_link_pressed: None,
            blinker: util::Blinker {
                items: HashMap::new(),
            },
            characters,
            stuffs,
            resources,
            builds,
            animated_corpses,
            builds_positions: HashMap::new(),
            link_button_ids: HashMap::new(),
            link_button_pressed: -1,
            move_requested: None,
            request_clicks,
            cursor_position: Point::new(0.0, 0.0),
            player_tile_id: String::from("PLAYER"),
            player_move_ticker: util::Ticker::new(15),
            top_bar,
            displaying_chat: false,
            display_chat_required: false,
            chat: None,
            chat_input_value: "".to_string(),
            previous_chat_button_state: fixed_button::State::new(),
            next_chat_button_state: fixed_button::State::new(),
            unread_conversation_id: vec![],
            replace_top_bar_start,
            replace_top_bar_by: None,
            send_quick_actions_transmitter,
            response_quick_actions_receiver,
        };
        zone_engine.update_link_button_data();
        zone_engine.update_builds_data();
        zone_engine
    }

    fn update_builds_data(&mut self) {
        for (_, build) in self.builds.iter() {
            let key = (build.row_i as i16, build.col_i as i16);
            if self.builds_positions.contains_key(&key) {
                self.builds_positions.get_mut(&key).unwrap().push(build.id);
            } else {
                self.builds_positions.insert(key, vec![build.id]);
            }
        }
    }

    fn update_link_button_data(&mut self) {
        let mut link_button_counter: i32 = 0;
        self.link_button_ids = HashMap::new();

        for item in self.resume_text.iter() {
            if let Some(_) = item.url {
                self.link_button_ids
                    .insert(item.name.clone(), link_button_counter);

                link_button_counter += 1;
            }
        }
    }

    fn proceed_quick_action_responses(&self) -> Vec<(String, TopBarMessageType)> {
        let mut messages: Vec<(String, TopBarMessageType)> = vec![];
        loop {
            match self.response_quick_actions_receiver.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(description) => {
                            let type_ = if description.type_ == "ERROR" {
                                TopBarMessageType::ERROR
                            } else {
                                TopBarMessageType::NORMAL
                            };
                            messages.push((
                                description
                                    .quick_action_response
                                    .unwrap_or("Erreur : Aucun message obtenu".to_string()),
                                type_,
                            ));
                        }
                        Err(error) => {
                            eprintln!("Error happens when make quick action : {}", error);
                            messages.push((
                                "Error happens when make quick action".to_string(),
                                TopBarMessageType::ERROR,
                            ));
                        }
                    };
                }
                Err(error) => match error {
                    crossbeam_channel::TryRecvError::Empty => break,
                    crossbeam_channel::TryRecvError::Disconnected => {
                        eprintln!("Error when reading quick action responses");
                        break;
                    }
                },
            }
        }

        messages
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
                    && self
                        .builds_positions
                        .contains_key(&(zone_row_i, zone_col_i))
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
                    self.sprite_i,
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
            &self.player_tile_id,
            self.get_real_x(self.player.x),
            self.get_real_y(self.player.y),
            self.sprite_i,
        ));

        for character in self.characters.values().into_iter() {
            if character.id == self.player.id {
                continue;
            }

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

            sprites.push(self.tile_sheet.create_sprite_for(
                "CHARACTER",
                real_x,
                real_y,
                self.sprite_i,
            ));
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
                    sprites.push(self.tile_sheet.create_sprite_for(
                        class,
                        real_x,
                        real_y,
                        self.sprite_i,
                    ));
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

            // TODO BS 20200722: use class system like for build and stuff
            if self.tile_sheet.have_id(&resource.id) {
                sprites.push(self.tile_sheet.create_sprite_for(
                    &resource.id,
                    real_x,
                    real_y,
                    self.sprite_i,
                ));
            } else {
                sprites.push(self.tile_sheet.create_sprite_for(
                    "RESOURCE_GENERIC",
                    real_x,
                    real_y,
                    self.sprite_i,
                ));
            }
        }

        sprites
    }

    fn get_build_sprites(&mut self, is_floor: bool) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];

        for build in self.builds.values().into_iter() {
            let real_x = self.get_real_x(build.position().1 as i16 * TILE_WIDTH);
            let real_y = self.get_real_y(build.position().0 as i16 * TILE_HEIGHT);
            if real_x < 0
                || real_x < START_SCREEN_X
                || real_x > self.end_screen_x
                || real_y < 0
                || real_y > self.end_screen_y
                || build.is_floor != is_floor
            {
                continue;
            }

            for class in build.get_classes().iter().rev() {
                if self.tile_sheet.have_id(class) {
                    sprites.push(self.tile_sheet.create_sprite_for(
                        class,
                        real_x,
                        real_y,
                        self.sprite_i,
                    ));
                    break;
                }
            }
        }

        sprites
    }

    fn get_animated_corpses(&mut self) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];

        for animated_corpse in self.animated_corpses.values().into_iter() {
            let real_x = self.get_real_x(animated_corpse.position().1 as i16 * TILE_WIDTH);
            let real_y = self.get_real_y(animated_corpse.position().0 as i16 * TILE_HEIGHT);
            if real_x < 0
                || real_x < START_SCREEN_X
                || real_x > self.end_screen_x
                || real_y < 0
                || real_y > self.end_screen_y
            {
                continue;
            }

            sprites.push(self.tile_sheet.create_sprite_for(
                &animated_corpse.type_,
                real_x,
                real_y,
                self.sprite_i,
            ));
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
                if let Some(corner) =
                    util::get_corner(&self.level, next_position.0, next_position.1)
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
                        return Some(MainMessage::ToDescriptionWithUrl {
                            url,
                            back_url: None,
                        });
                    }
                }
            }
        }

        None
    }

    fn receive_new_top_bar_message(
        &mut self,
        message: String,
        type_: TopBarMessageType,
        destroy_previous: bool,
    ) {
        let text_color = match type_ {
            TopBarMessageType::NORMAL => Color::WHITE,
            TopBarMessageType::ERROR => Color::RED,
        };
        let previous_top_bar = if let Some(replace_top_bar_by) = self.replace_top_bar_by.as_ref() {
            Some(replace_top_bar_by.clone())
        } else if let Some(top_bar) = self.top_bar.as_ref() {
            Some(top_bar.clone())
        } else {
            None
        };

        self.top_bar = Some(TopBar {
            text: message,
            text_color,
            display_buttons: false,
            on_click: None,
        });
        self.replace_top_bar_start = Some(SystemTime::now());
        if !destroy_previous {
            self.replace_top_bar_by = previous_top_bar;
        }
    }

    fn there_is_build_not_browseable(&self, row_i: i16, col_i: i16) -> bool {
        if let Some(build_ids) = self.builds_positions.get(&(row_i, col_i)) {
            for build_id in build_ids.iter() {
                // TODO BS: transport type
                if !self
                    .builds
                    .get(build_id)
                    .unwrap()
                    .traversable
                    .get("WALKING")
                    .unwrap_or(&true)
                {
                    return true;
                }
            }
        }
        false
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
        let tile_is_browseable = self.tiles.browseable(&next_tile_id);
        let build_not_browseable =
            self.there_is_build_not_browseable(try_next_tile.0, try_next_tile.1);

        if tile_is_browseable && !build_not_browseable {
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

        if player_from_top_left_cols < BORDERS_TO_SEE_PLAYER_LEN {
            self.start_zone_col_i -= 1;
        }
        if player_from_top_left_rows < BORDERS_TO_SEE_PLAYER_LEN {
            self.start_zone_row_i -= 1;
        }
        if player_from_bottom_right_cols < BORDERS_TO_SEE_PLAYER_LEN {
            self.start_zone_col_i += 1;
        }
        if player_from_bottom_right_rows < BORDERS_TO_SEE_PLAYER_LEN {
            self.start_zone_row_i += 1;
        }
    }

    fn xy_to_zone_coords(&self, x: i16, y: i16) -> (i16, i16) {
        let x_from_start_screen = x - START_SCREEN_X;
        let y_from_start_screen = y - START_SCREEN_Y;
        let absolute_row_i = y_from_start_screen / TILE_HEIGHT;
        let absolute_col_i = x_from_start_screen / TILE_WIDTH;
        let row_i = absolute_row_i + self.start_zone_row_i;
        let col_i = absolute_col_i + self.start_zone_col_i;

        (row_i, col_i)
    }

    fn get_move_modifier_for_around(
        &self,
        current_row_i: i16,
        current_col_i: i16,
        row_i: i16,
        col_i: i16,
    ) -> (i16, i16) {
        if current_row_i - 1 == row_i && current_col_i - 1 == col_i {
            return (-1, -1);
        }

        if current_row_i - 1 == row_i && current_col_i == col_i {
            return (0, -1);
        }

        if current_row_i - 1 == row_i && current_col_i + 1 == col_i {
            return (1, -1);
        }

        if current_row_i == row_i && current_col_i - 1 == col_i {
            return (-1, 0);
        }

        if current_row_i == row_i && current_col_i + 1 == col_i {
            return (1, 0);
        }

        if current_row_i == row_i && current_col_i == col_i {
            return (0, 0);
        }

        if current_row_i + 1 == row_i && current_col_i - 1 == col_i {
            return (-1, 1);
        }

        if current_row_i + 1 == row_i && current_col_i == col_i {
            return (0, 1);
        }

        if current_row_i + 1 == row_i && current_col_i + 1 == col_i {
            return (1, 1);
        }

        eprintln!("Around position must used !");
        (0, 0)
    }

    fn apply_chat_text_buffer(&mut self, text_buffer: String) {
        for c in text_buffer.chars() {
            match c {
                // Match ASCII backspace and delete from the text buffer
                '\u{8}' => {
                    self.chat_input_value.pop();
                }
                // Tabulation | Enter | Escape
                '\t' | '\r' | '\u{1b}' => {}
                _ => {
                    self.chat_input_value.push(c);
                }
            }
        }
    }
}

impl Drop for ZoneEngine {
    fn drop(&mut self) {
        self.socket.close().unwrap();
    }
}

impl Engine for ZoneEngine {
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer, _illustration: Option<Image>) {
        frame.clear(Color::BLACK);
        let mut sprites: Vec<Sprite> = vec![];

        sprites.extend(self.get_zone_sprites(Some(self.level.world_tile_type_id.clone())));
        sprites.extend(self.get_zone_sprites(None));
        sprites.extend(self.get_build_sprites(true));
        sprites.extend(self.get_build_sprites(false));
        sprites.extend(self.get_stuff_sprites());
        sprites.extend(self.get_resource_sprites());
        sprites.extend(self.get_animated_corpses());
        sprites.extend(self.get_characters_sprites());
        sprites.push(Sprite {
            source: Rectangle {
                x: 1100,
                y: 0,
                width: 120,
                height: 225,
            },
            position: Point::new(frame.width() - RIGHT_MENU_WIDTH as f32 - 5.0, 0.0),
            scale: (1.0, 1.35),
        });

        if let Some(request_clicks) = &self.request_clicks {
            let cursor_x = self.cursor_position.x.round() as i16;
            let cursor_y = self.cursor_position.y.round() as i16;
            if cursor_x > START_SCREEN_X && cursor_y > START_SCREEN_Y {
                for class in request_clicks.cursor_classes.iter().rev() {
                    if self.tile_sheet.have_id(class) {
                        let (cursor_row_i, cursor_col_i) =
                            self.xy_to_zone_coords(cursor_x, cursor_y);
                        let real_x = self.get_real_x(cursor_col_i * TILE_WIDTH);
                        let real_y = self.get_real_y(cursor_row_i * TILE_HEIGHT);

                        sprites.push(self.tile_sheet.create_sprite_for(
                            class,
                            real_x,
                            real_y,
                            self.sprite_i,
                        ));
                        break;
                    }
                }
            }
        }

        self.tile_sheet_batch.clear();
        self.tile_sheet_batch.extend(sprites);
        self.tile_sheet_batch.draw(&mut frame.as_target());

        if let Some(hover_character_id) = &self.hover_character_id {
            if let Some(character) = self.characters.get(hover_character_id) {
                let avatar_uuid =
                    if character.avatar_is_validated && character.avatar_uuid.is_some() {
                        if let Some(avatar_uuid) = &character.avatar_uuid {
                            avatar_uuid.to_string()
                        } else {
                            "0000".to_string()
                        }
                    } else {
                        "0000".to_string()
                    };

                let real_x = self.get_real_x(character.position().1 as i16 * TILE_WIDTH);
                let real_y = self.get_real_y(character.position().0 as i16 * TILE_HEIGHT);

                if let Some((avatar_batch, width, height)) = self.avatars.get_mut(&avatar_uuid) {
                    avatar_batch.clear();
                    avatar_batch.add(Sprite {
                        source: Rectangle {
                            x: 0,
                            y: 0,
                            width: *width,
                            height: *height,
                        },
                        position: Point::new(
                            real_x as f32 - (TILE_WIDTH as f32 / 2.0),
                            real_y as f32 - (TILE_HEIGHT as f32 + 16.0),
                        ),
                        scale: (1.0, 1.0),
                    });
                    avatar_batch.draw(&mut frame.as_target());
                }
            }
        }

        let player_avatar_uuid = if self.player.avatar_is_validated {
            if let Some(avatar_uuid) = &self.player.avatar_uuid {
                avatar_uuid.clone()
            } else {
                "0000".to_string()
            }
        } else {
            "0000".to_string()
        };
        if let Some((avatar_batch, width, height)) = self.avatars.get_mut(&player_avatar_uuid) {
            avatar_batch.clear();
            avatar_batch.add(Sprite {
                source: Rectangle {
                    x: 0,
                    y: 0,
                    width: *width,
                    height: *height,
                },
                position: Point::new(frame.width() as f32 - 32.0 - *width as f32, 5.0),
                scale: (1.0, 1.0),
            });
            avatar_batch.draw(&mut frame.as_target());
        }
    }

    fn update(&mut self, window: &Window) -> Option<MainMessage> {
        self.i += 1;
        if self.i % 10 == 0 {
            self.sprite_i += 1;
        }
        if self.sprite_i >= 6 {
            self.sprite_i = 0;
        }

        self.end_screen_x = window.width() as i16;
        self.end_screen_y = window.height() as i16;
        self.update_zone_display();

        if let Some(replace_top_bar_start) = self.replace_top_bar_start.as_mut() {
            if replace_top_bar_start.elapsed().unwrap() > Duration::from_secs(3) {
                if let Some(replace_top_bar_by) = &self.replace_top_bar_by {
                    self.top_bar = Some(replace_top_bar_by.clone());
                } else {
                    self.top_bar = None;
                };
                self.replace_top_bar_start = None;
                self.replace_top_bar_by = None;
            }
        }

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
                            // FIXME BS NOW : avatar_uuid & avatar_is_validated,
                            avatar_uuid: None,
                            avatar_is_validated: false,
                        },
                    );
                }
                ZoneEventType::CharacterExit { character_id } => {
                    if let None = self.characters.remove(&character_id) {
                        if &character_id == &self.player.id {
                            println!("Reload zone because player left on server side");
                            return Some(MainMessage::DescriptionToZone {
                                request_clicks: None,
                            });
                        } else {
                            println!(
                                "{} left zone but was not in list of characters",
                                &character_id
                            );
                        }
                    } else {
                        println!("{} exit from zone", &character_id);
                    }
                }
                ZoneEventType::ThereIsAround {
                    stuff_count,
                    resource_count,
                    build_count,
                    character_count,
                    quick_actions,
                } => {
                    self.around_items_count = stuff_count + resource_count;
                    self.around_builds_count = build_count;
                    self.around_characters_count = character_count;
                    self.around_quick_actions = quick_actions;
                }
                ZoneEventType::NewResumeText { resume } => {
                    self.resume_text = resume;
                    self.update_link_button_data();
                }
                ZoneEventType::NewBuild { build } => {
                    self.builds.insert(build.id, build);
                    self.update_builds_data();
                }
                ZoneEventType::ZoneTileReplace {
                    row_i,
                    col_i,
                    new_tile_id,
                } => {
                    println!("Replace tile at {}:{} with {}", row_i, col_i, &new_tile_id);
                    self.level.rows[row_i as usize].cols[col_i as usize] = new_tile_id;
                }
                ZoneEventType::AnimatedCorpseMove {
                    to_row_i,
                    to_col_i,
                    animated_corpse_id,
                } => {
                    if let Some(mut moved_animated_corpse) =
                        self.animated_corpses.get_mut(&animated_corpse_id)
                    {
                        moved_animated_corpse.zone_row_i = to_row_i;
                        moved_animated_corpse.zone_col_i = to_col_i;
                    } else {
                        eprintln!("Unknown animated corpse {} for move", animated_corpse_id)
                    }
                }
                ZoneEventType::NewChatMessage {
                    character_id: _,
                    conversation_id,
                    conversation_title,
                    message,
                } => {
                    println!(
                        "New chat message received: {} ({:?})",
                        message, conversation_id
                    );
                    match self.chat.as_mut() {
                        Some(chat) => {
                            if chat.conversation_id == conversation_id {
                                chat.messages.push(message);
                            } else if self.display_chat_required {
                                self.display_chat_required = false;
                                if let Some(conversation_title) = conversation_title {
                                    self.top_bar.as_mut().unwrap().text = conversation_title;
                                };
                                chat.messages = vec![message];
                                chat.conversation_id = conversation_id;
                            };
                            if chat.messages.len() > CHAT_MESSAGE_COUNT as usize {
                                chat.messages.remove(0);
                            };
                        }
                        None => {
                            println!("open chat box");
                            if self.display_chat_required {
                                self.chat = Some(Chat {
                                    messages: vec![message],
                                    conversation_id,
                                });
                                self.top_bar = Some(TopBar {
                                    text: conversation_title
                                        .unwrap_or("Chat de la zone".to_string()),
                                    text_color: Color::WHITE,
                                    display_buttons: true,
                                    on_click: Some(Message::DismissChat),
                                });
                                self.display_chat_required = false;
                                self.displaying_chat = true;
                            } else {
                                if !self.request_clicks.is_some() {
                                    match self.top_bar.as_mut() {
                                        Some(top_bar) => {
                                            top_bar.text = message;
                                        }
                                        None => {
                                            self.top_bar = Some(TopBar {
                                                text: message,
                                                text_color: Color::WHITE,
                                                display_buttons: false,
                                                on_click: Some(Message::RequestChat(
                                                    conversation_id,
                                                )),
                                            });
                                        }
                                    }
                                    self.replace_top_bar_start = Some(SystemTime::now());
                                    self.unread_conversation_id.push(conversation_id);
                                } else {
                                    self.unread_conversation_id.push(conversation_id);
                                }
                            }
                        }
                    }
                }
                ZoneEventType::TopBarMessage { message, type_ } => {
                    self.receive_new_top_bar_message(message, type_, false);
                }
                _ => println!("unknown event type {:?}", &event.event_type),
            }
        }

        let messages = self.proceed_quick_action_responses();

        for (msg, type_) in messages {
            self.receive_new_top_bar_message(msg, type_, true);
        }

        None
    }

    fn interact(&mut self, input: &mut MyGameInput, window: &mut Window) -> Option<MainMessage> {
        let mut try_player_moves: Vec<(i16, i16)> = vec![];
        self.cursor_position = input.cursor_position.clone();

        if input.mouse_buttons_pressed.contains(&mouse::Button::Left) {
            let click_x = input.cursor_position.x.round() as i16;
            let click_y = input.cursor_position.y.round() as i16;
            let (to_row_i, to_col_i) = self.xy_to_zone_coords(click_x, click_y);
            input.mouse_buttons_pressed.clear();

            // Is that a move/requested click ? Must not be in menu
            if click_x > START_SCREEN_X
                && click_y > START_SCREEN_Y
                && click_y < (window.height() - QUICK_ACTION_ROW_HEIGHT as f32) as i16
            {
                if let Some(request_clicks) = &self.request_clicks {
                    // REQUESTED CLICK
                    self.socket.send(event::ZoneEvent {
                        event_type_name: String::from(event::CLICK_ACTION_EVENT),
                        event_type: event::ZoneEventType::ClickActionEvent {
                            action_type: request_clicks.action_type.clone(),
                            action_description_id: request_clicks.action_description_id.clone(),
                            row_i: to_row_i,
                            col_i: to_col_i,
                        },
                    });

                    if !request_clicks.many {
                        self.request_clicks = None;
                    }
                } else {
                    // MOVE
                    let player_position =
                        (self.player.position.0 as i16, self.player.position.1 as i16);
                    if let Some(result) = astar(
                        &player_position,
                        |(row_i, col_i)| {
                            self.level
                                .get_successors(&self.tiles, row_i.clone(), col_i.clone())
                        },
                        |(row_i, col_i)| {
                            ((absdiff(row_i.clone(), to_row_i) + absdiff(col_i.clone(), to_col_i))
                                as u32)
                                / 3
                        },
                        |(row_i, col_i)| (row_i.clone(), col_i) == (to_row_i, &to_col_i),
                    ) {
                        let mut moves = vec![];
                        let mut current_position = player_position.clone();
                        self.player.x = current_position.1 * TILE_HEIGHT;
                        self.player.y = current_position.0 * TILE_WIDTH;
                        for next_move in result.0 {
                            let modifier = self.get_move_modifier_for_around(
                                current_position.0,
                                current_position.1,
                                next_move.0,
                                next_move.1,
                            );
                            // FIXME BS: Can work only with tile squares !
                            for _ in 0..TILE_WIDTH {
                                moves.push(modifier.clone());
                            }
                            current_position = next_move;
                        }
                        self.move_requested = Some(moves);
                    } else {
                        println!("Impossible move");
                    }
                }
            }
        }

        if !input.keys_pressed.is_empty() {
            if self.chat.is_some() {
                self.apply_chat_text_buffer(input.text_buffer.clone());
                input.text_buffer = String::new();
            }
            let move_modifier = if input.keys_pressed.contains(&keyboard::KeyCode::LShift)
                || input.keys_pressed.contains(&keyboard::KeyCode::RShift)
            {
                3
            } else {
                1
            };
            if input.keys_pressed.contains(&keyboard::KeyCode::Right) {
                try_player_moves.push((move_modifier, 0));
                self.move_requested = None;
                self.player_tile_id = String::from("PLAYER");
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Left) {
                try_player_moves.push((-move_modifier, 0));
                self.move_requested = None;
                self.player_tile_id = String::from("PLAYER_LEFT");
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Up) {
                try_player_moves.push((0, -move_modifier));
                self.move_requested = None;
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Down) {
                try_player_moves.push((0, move_modifier));
                self.move_requested = None;
            }
        }

        match input.key_code {
            Some(keyboard::KeyCode::Escape) => {
                input.key_code = None;

                if self.request_clicks.is_some() {
                    self.request_clicks = None;
                    self.top_bar = None;
                } else if self.displaying_chat {
                    self.top_bar = None;
                    self.chat = None;
                    self.displaying_chat = false;
                } else {
                    return Some(MainMessage::ToExit);
                }
            }
            Some(keyboard::KeyCode::Return) => {
                if !self.displaying_chat && !self.display_chat_required {
                    self.display_chat_required = true;
                    self.socket.send(event::ZoneEvent {
                        event_type_name: String::from(event::REQUEST_CHAT),
                        event_type: event::ZoneEventType::RequestChat {
                            character_id: String::from(self.player.id.as_str()),
                            previous_conversation_id: None,
                            message_count: CHAT_MESSAGE_COUNT,
                            next: false,
                            previous: false,
                        },
                    });
                } else if self.displaying_chat {
                    println!("send chat message {}", &self.chat_input_value);
                    self.socket.send(event::ZoneEvent {
                        event_type_name: String::from(event::NEW_CHAT_MESSAGE),
                        event_type: event::ZoneEventType::NewChatMessage {
                            character_id: self.player.id.clone(),
                            conversation_id: self.chat.as_ref().unwrap().conversation_id,
                            conversation_title: None,
                            message: self.chat_input_value.clone(),
                        },
                    });
                    self.chat_input_value = "".to_string();
                    input.key_code = None;
                }
            }
            _ => {}
        }

        if let Some(main_message) = self.try_travel(&try_player_moves) {
            return Some(main_message);
        }

        if self.player_move_ticker.tick() {
            if let Some(move_requested) = self.move_requested.as_ref() {
                if let Some(next_move) = move_requested.iter().next() {
                    try_player_moves.push(*next_move);
                    self.move_requested.as_mut().unwrap().remove(0);
                } else {
                    self.move_requested = None;
                }
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
                    self.around_items_count = 0;
                    self.around_builds_count = 0;
                    self.around_characters_count = 0;
                    self.around_quick_actions = vec![];
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
            if !player_have_move.0 {
                self.move_requested = None;
            }
        }

        while let Some(avatar_to_load) = self.avatars_to_load.pop() {
            match graphics::Image::new(
                window.gpu(),
                format!("cache/character_avatar__zone_thumb__{}.png", avatar_to_load),
            ) {
                Ok(image) => {
                    let width = image.width();
                    let height = image.height();
                    let batch = Batch::new(image);
                    self.avatars
                        .insert(avatar_to_load.clone(), (batch, width, height));
                }
                Err(error) => {
                    eprintln!("Error when loading avatar {}: {}", avatar_to_load, error);
                }
            };
        }

        let mut found = false;
        for (character_id, character) in &self.characters {
            let real_x = self.get_real_x(character.position().1 as i16 * TILE_WIDTH) as f32;
            let real_y = self.get_real_y(character.position().0 as i16 * TILE_HEIGHT) as f32;

            if input.cursor_position.x > real_x
                && input.cursor_position.x <= (real_x + TILE_WIDTH as f32)
                && input.cursor_position.y > real_y
                && input.cursor_position.y <= (real_y + TILE_HEIGHT as f32)
            {
                found = true;
                self.hover_character_id = Some(character_id.clone());
            }
        }

        if !found {
            self.hover_character_id = None;
        }

        None
    }

    fn react(&mut self, event: Message, _window: &mut Window) -> Option<MainMessage> {
        match event {
            Message::WorldMenuButtonPressed => return Some(MainMessage::ToWorld),
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
            Message::ServerInfosMenuButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: "/system/describe/infos".to_string(),
                    back_url: None,
                })
            }
            Message::OpenAccountButtonPressed => {
                let url = format!("{}/account/manage", self.server.address);
                println!("Open url {} with web browser", url);
                if webbrowser::open(&url).is_ok() {
                    // ..
                }
            }
            Message::ExitMenuButtonPressed => return Some(MainMessage::ToExit),
            Message::LinkButtonPressed(id) => {
                self.link_button_pressed = id;
            }
            Message::LinkButtonReleased(url) => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: url.clone(),
                    back_url: None,
                });
            }
            Message::AroundItemsButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!(
                        "/character/{}/action/TRANSFER_GROUND/TRANSFER_GROUND",
                        self.player.id
                    )
                    .to_string(),
                    back_url: None,
                })
            }
            Message::AroundBuildButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/character/{}/describe_around_builds", self.player.id)
                        .to_string(),
                    back_url: None,
                })
            }
            Message::AroundCharactersButtonPressed => {
                return Some(MainMessage::ToDescriptionWithUrl {
                    url: format!("/character/{}/describe_around_characters", self.player.id)
                        .to_string(),
                    back_url: None,
                })
            }
            Message::PreviousChatButtonPressed => {
                self.chat.as_mut().unwrap().messages = vec![];
                self.display_chat_required = true;
                self.socket.send(event::ZoneEvent {
                    event_type_name: String::from(event::REQUEST_CHAT),
                    event_type: event::ZoneEventType::RequestChat {
                        character_id: String::from(self.player.id.as_str()),
                        previous_conversation_id: self.chat.as_ref().unwrap().conversation_id,
                        message_count: CHAT_MESSAGE_COUNT,
                        next: false,
                        previous: true,
                    },
                });
            }
            Message::NextChatButtonPressed => {
                self.chat.as_mut().unwrap().messages = vec![];
                self.display_chat_required = true;
                self.socket.send(event::ZoneEvent {
                    event_type_name: String::from(event::REQUEST_CHAT),
                    event_type: event::ZoneEventType::RequestChat {
                        character_id: String::from(self.player.id.as_str()),
                        previous_conversation_id: self.chat.as_ref().unwrap().conversation_id,
                        message_count: CHAT_MESSAGE_COUNT,
                        next: true,
                        previous: false,
                    },
                });
            }
            Message::DismissChat => {
                self.chat = None;
                self.top_bar = None;
                self.displaying_chat = false;
                self.display_chat_required = false;
                self.chat_input_value = "".to_string();
                self.unread_conversation_id = vec![];
                self.replace_top_bar_start = None;
            }
            Message::DismissRequestClicks => {
                self.request_clicks = None;
            }
            Message::RequestChat(conversation_id) => {
                self.display_chat_required = true;
                self.replace_top_bar_start = None;
                self.top_bar = None;
                self.socket.send(event::ZoneEvent {
                    event_type_name: String::from(event::REQUEST_CHAT),
                    event_type: event::ZoneEventType::RequestChat {
                        character_id: String::from(self.player.id.as_str()),
                        previous_conversation_id: conversation_id,
                        message_count: CHAT_MESSAGE_COUNT,
                        next: false,
                        previous: false,
                    },
                });
            }
            Message::QuickActionPressed(link) => {
                self.current_quick_action_link_pressed = Some(link);
            }
            Message::QuickActionReleased(link) => {
                self.current_quick_action_link_pressed = None;
                match self.send_quick_actions_transmitter.send(link) {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("Error when send link to quick actions channel: {}", error)
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn layout(&mut self, window: &Window, _illustration: Option<Image>) -> Element {
        let event_class = if self.player.unread_event && self.blinker.visible(500, 'E') {
            thin_button::Class::Primary
        } else {
            thin_button::Class::Secondary
        };
        let business_class = if self.player.unread_transactions && self.blinker.visible(500, 'B') {
            thin_button::Class::Primary
        } else {
            thin_button::Class::Secondary
        };
        let affinity_class =
            if self.player.unvote_affinity_relation && self.blinker.visible(500, 'A') {
                thin_button::Class::Primary
            } else {
                thin_button::Class::Secondary
            };
        let zone_message_class =
            if self.player.unread_zone_message && self.blinker.visible(500, 'M') {
                thin_button::Class::Primary
            } else {
                thin_button::Class::Secondary
            };
        let conversation_class =
            if self.player.unread_conversation && self.blinker.visible(500, 'C') {
                thin_button::Class::Primary
            } else {
                thin_button::Class::Secondary
            };
        let actions_class = if self.player.pending_actions != 0 {
            thin_button::Class::Primary
        } else {
            thin_button::Class::Secondary
        };
        let actions_label = if self.player.pending_actions != 0 {
            format!("Actions({})", self.player.pending_actions)
        } else {
            "Actions".to_string()
        };

        let left_menu = Column::new()
            .width(LEFT_MENU_WIDTH as u32)
            .height(window.height() as u32 - QUICK_ACTION_ROW_HEIGHT)
            // .align_items(Align::Center)
            // .justify_content(Justify::Center)
            .padding(0)
            .spacing(5)
            .push(
                Button::new(&mut self.world_menu_button_state, "Carte du monde")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::WorldMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.card_menu_button_state, "Fiche")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::CardMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.events_menu_button_state, "Événements")
                    .class(event_class)
                    .on_press(Message::EventsMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.business_menu_button_state, "Commerce")
                    .class(business_class)
                    .on_press(Message::BusinessMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.affinities_menu_button_state, "Affinités")
                    .class(affinity_class)
                    .on_press(Message::AffinitiesMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.zone_menu_button_state, "Zone")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::ZoneMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.zone_messages_menu_button_state, "Chat")
                    .class(zone_message_class)
                    .on_press(Message::ZoneMessagesMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.conversations_menu_button_state, "Conversations")
                    .class(conversation_class)
                    .on_press(Message::ConversationsMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.inventory_menu_button_state, "Inventaire")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::InventoryMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.action_menu_button_state, &actions_label)
                    .class(actions_class)
                    .on_press(Message::ActionMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.build_menu_button_state, "Construire")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::BuildMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.info_menu_button_state, "Infos serveur")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::ServerInfosMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.account_button_state, "Mon compte")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::OpenAccountButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.exit_menu_button_state, "Quitter")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::ExitMenuButtonPressed)
                    .width(175),
            );

        let mut right_menu = Column::new()
            .width(RIGHT_MENU_WIDTH as u32)
            .height(window.height() as u32 - QUICK_ACTION_ROW_HEIGHT)
            .push(Row::new().height(76))
            .push(
                Row::new().push(
                    text::Text::new(&self.player.name)
                        .color(Color::BLACK)
                        .horizontal_alignment(HorizontalAlignment::Center),
                ),
            );

        let mut right_column_1 = Column::new().width(30);
        let mut right_column_2 = Column::new().width(40);
        for item in self.resume_text.iter() {
            let name_icon = match item.name.as_str() {
                "PV" => icon::Icon::new(icon::Class::Heart),
                "PA" => icon::Icon::new(icon::Class::Time),
                "Faim" => icon::Icon::new(icon::Class::Ham),
                "Soif" => icon::Icon::new(icon::Class::Water),
                "A boire" => icon::Icon::new(icon::Class::AnyWater),
                "A manger" => icon::Icon::new(icon::Class::AnyHam),
                "Fatigue" => icon::Icon::new(icon::Class::Tiredness),
                "Suivis" => icon::Icon::new(icon::Class::Followed),
                "Suiveurs" => icon::Icon::new(icon::Class::Follower),
                "Combattants" => icon::Icon::new(icon::Class::Shield),
                _ => icon::Icon::new(icon::Class::Empty),
            };
            right_column_1 = right_column_1.push(name_icon);
            if item.value_is_str {
                let value = item.value_str.as_ref().unwrap();
                if value.eq("Oui") {
                    right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Ok));
                } else if value.eq("Non") {
                    right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Ko));
                } else if value.eq("Ok") {
                    right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Health1));
                } else if value.eq("Moyen") {
                    right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Health2));
                } else if value.eq("Mauvais") {
                    right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Health3));
                } else if value.eq("Critique") {
                    if self.blinker.visible(250, 'x') {
                        right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Health4));
                    } else {
                        right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Empty));
                    };
                } else if value.eq("Faible") {
                    right_column_2 = right_column_2.push(icon::Icon::new(icon::Class::Warning));
                } else {
                    right_column_2 =
                        right_column_2.push(text::Text::new(value).color(Color::BLACK));
                }
            }
            if item.value_is_float {
                if contains_string(&item.classes, "inverted_percent")
                    || contains_string(&item.classes, "percent")
                {
                    let progress = if contains_string(&item.classes, "inverted_percent") {
                        (100.0 - item.value_float.unwrap()) / 100.0
                    } else {
                        item.value_float.unwrap() / 100.0
                    };

                    let color_class = if contains_string(&item.classes, "yellow") {
                        progress_bar::ColorClass::Yellow
                    } else if contains_string(&item.classes, "red") {
                        progress_bar::ColorClass::Red
                    } else {
                        progress_bar::ColorClass::Green
                    };
                    if color_class == progress_bar::ColorClass::Red {
                        if self.blinker.visible(250, 'h') {
                            right_column_2 = right_column_2.push(
                                progress_bar::ProgressBar::new(
                                    progress,
                                    progress_bar::Class::SimpleThin,
                                    color_class,
                                )
                                .width(40),
                            );
                        } else {
                            right_column_2 =
                                right_column_2.push(icon::Icon::new(icon::Class::Empty));
                        }
                    } else {
                        right_column_2 = right_column_2.push(
                            progress_bar::ProgressBar::new(
                                (100.0 - item.value_float.unwrap()) / 100.0,
                                progress_bar::Class::SimpleThin,
                                color_class,
                            )
                            .width(40),
                        );
                    }
                } else {
                    right_column_2 = right_column_2.push(
                        text::Text::new(&item.value_float.as_ref().unwrap().clone().to_string())
                            .color(Color::BLACK),
                    );
                }
            }
        }
        let right_menu_row = Row::new().push(right_column_1).push(right_column_2);
        right_menu = right_menu.push(right_menu_row);

        if self.around_items_count > 0 {
            right_menu = right_menu.push(
                fixed_button::Button::new(
                    &mut self.around_items_button_state,
                    &self.around_items_count.to_string(),
                )
                .on_press(Message::AroundItemsButtonPressed)
                .class(fixed_button::Class::Item),
            )
        }

        if self.around_builds_count > 0 {
            right_menu = right_menu.push(
                fixed_button::Button::new(
                    &mut self.around_builds_button_state,
                    &self.around_builds_count.to_string(),
                )
                .on_press(Message::AroundBuildButtonPressed)
                .class(fixed_button::Class::Build),
            )
        }

        if self.around_characters_count > 0 {
            right_menu = right_menu.push(
                fixed_button::Button::new(
                    &mut self.around_characters_button_state,
                    &self.around_characters_count.to_string(),
                )
                .on_press(Message::AroundCharactersButtonPressed)
                .class(fixed_button::Class::Character),
            )
        }

        let mut quick_actions_row = Row::new()
            .align_items(Align::Center)
            .width(window.width() as u32);
        for quick_action in &self.around_quick_actions {
            let pressed = if let Some(pressed_link) = &self.current_quick_action_link_pressed {
                quick_action.link == *pressed_link
            } else {
                false
            };
            quick_actions_row = quick_actions_row.push(SheetButton::new(
                pressed,
                &self.tile_sheet,
                &quick_action.classes1,
                &quick_action.classes2,
                message::Message::QuickActionPressed(quick_action.link.clone()),
                Message::QuickActionReleased(quick_action.link.clone()),
            ))
        }

        let mut center_column = Column::new()
            .width(
                window.width() as u32
                    - LEFT_MENU_WIDTH as u32
                    - RIGHT_MENU_WIDTH as u32
                    - MARGIN_RIGHT_CHAT,
            )
            .height(window.height() as u32 - QUICK_ACTION_ROW_HEIGHT);

        if let Some(top_bar) = self.top_bar.as_ref() {
            let mut top_bar_row = Row::new().height(fixed_button::NODE_HEIGHT);
            if top_bar.display_buttons {
                top_bar_row = top_bar_row.push(
                    Column::new()
                        .push(
                            fixed_button::Button::new(&mut self.previous_chat_button_state, "")
                                .on_press(Message::PreviousChatButtonPressed)
                                .class(fixed_button::Class::Back),
                        )
                        .width(TOP_BAR_BUTTON_WIDTH),
                );
            }

            if let Some(message) = top_bar.on_click.as_ref() {
                top_bar_row = top_bar_row.push(
                    Column::new()
                        .push(
                            Link::new(
                                false,
                                &top_bar.text,
                                message.clone(),
                                message.clone(),
                                Some(text::Class::BgGray2),
                            )
                            .fill_width()
                            .height(fixed_button::NODE_HEIGHT)
                            .vertical_alignment(VerticalAlignment::Center)
                            .horizontal_alignment(HorizontalAlignment::Center),
                        )
                        .height(fixed_button::NODE_HEIGHT),
                );
            } else {
                top_bar_row = top_bar_row.push(
                    Column::new()
                        .push(
                            text::Text::new(&top_bar.text)
                                .color(top_bar.text_color)
                                .class(Some(text::Class::BgGray2))
                                .height(fixed_button::NODE_HEIGHT)
                                .vertical_alignment(VerticalAlignment::Center)
                                .horizontal_alignment(HorizontalAlignment::Center),
                        )
                        .height(fixed_button::NODE_HEIGHT),
                );
            };

            if top_bar.display_buttons {
                top_bar_row = top_bar_row.push(
                    Column::new()
                        .push(
                            fixed_button::Button::new(&mut self.next_chat_button_state, "")
                                .on_press(Message::NextChatButtonPressed)
                                .class(fixed_button::Class::Next),
                        )
                        .width(TOP_BAR_BUTTON_WIDTH),
                );
            }

            center_column = center_column.push(top_bar_row);
        };

        if let Some(chat) = self.chat.as_ref() {
            for message in &chat.messages {
                center_column = center_column.push(
                    text::Text::new(message)
                        .class(Some(text::Class::BgGray3))
                        .height(CHAT_LINE_HEIGHT),
                );
            }
            let blink_char = if self.blinker.visible(250, 'c') {
                Some('_')
            } else {
                None
            };
            center_column = center_column.push(TextInput::new(
                TEXT_INPUT_CHAT_ID,
                "Parler",
                &self.chat_input_value,
                Message::TextInputSelected,
                blink_char,
                Some(text::Class::BgGray1),
            ));
        }

        let layout = Row::new().push(
            Column::new()
                .push(
                    Row::new()
                        .push(left_menu)
                        .push(center_column)
                        .push(Column::new().width(MARGIN_RIGHT_CHAT))
                        .push(right_menu)
                        .align_items(Align::Stretch)
                        .height(window.height() as u32 - QUICK_ACTION_ROW_HEIGHT),
                )
                // Bottom
                .push(quick_actions_row),
        );

        layout.into()
    }

    fn teardown(&mut self) {
        // TODO: manage case where fail to close
        self.socket.close().unwrap();
    }
}
