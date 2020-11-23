use crate::engine::Engine;
use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::player::Player;
use crate::entity::resource::Resource;
use crate::entity::stuff::Stuff;
use crate::event::ZoneEventType;
use crate::game::{TILE_HEIGHT, TILE_WIDTH};
use crate::gui::lang::model::RequestClicks;
use crate::input::MyGameInput;
use crate::level::Level;
use crate::message::{MainMessage, Message};
use crate::server::Server;
use crate::sheet::TileSheet;
use crate::socket::ZoneSocket;
use crate::tile::zone::Tiles;
use crate::ui::widget::state_less_button;
use crate::ui::widget::state_less_button::StateLessButton;
use crate::ui::widget::text::Text;
use crate::ui::widget::thin_button;
use crate::ui::widget::thin_button::Button;
use crate::ui::Column;
use crate::ui::Element;
use crate::ui::Row;
use crate::{event, util};
use coffee::graphics::{Batch, Color, Frame, Point, Sprite, Window};
use coffee::input::keyboard;
use coffee::input::mouse;
use coffee::ui::Align;
use coffee::{graphics, Timer};
use pathfinding::prelude::{absdiff, astar};
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
    exit_menu_button_state: thin_button::State,
    resume_text: Vec<(String, Option<String>)>,
    around_text: Vec<(String, Option<String>)>,
    around_wait: Option<Instant>,
    menu_blinker: util::Blinker<char>,
    characters: HashMap<String, Character>,
    stuffs: HashMap<String, Stuff>,
    resources: Vec<Resource>,
    builds: HashMap<i32, Build>,
    builds_positions: HashMap<(i16, i16), Vec<i32>>,
    link_button_ids: HashMap<String, i32>,
    link_button_pressed: i32,
    move_requested: Option<Vec<(i16, i16)>>,
    request_clicks: Option<RequestClicks>,
    cursor_position: Point,
    player_tile_id: String,
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
        builds: HashMap<i32, Build>,
        request_clicks: Option<RequestClicks>,
    ) -> Self {
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
            exit_menu_button_state: thin_button::State::new(),
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
            builds_positions: HashMap::new(),
            link_button_ids: HashMap::new(),
            link_button_pressed: -1,
            move_requested: None,
            request_clicks,
            cursor_position: Point::new(0.0, 0.0),
            player_tile_id: String::from("PLAYER"),
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

        for (text, url) in self.resume_text.iter() {
            if let Some(_) = url {
                self.link_button_ids
                    .insert(text.clone(), link_button_counter);

                link_button_counter += 1;
            }
        }

        for (text, url) in self.around_text.iter() {
            if let Some(_) = url {
                self.link_button_ids
                    .insert(text.clone(), link_button_counter);

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

            // TODO BS 20200722: use class system like for build and stuff
            if self.tile_sheet.have_id(&resource.id) {
                sprites.push(
                    self.tile_sheet
                        .create_sprite_for(&resource.id, real_x, real_y),
                );
            } else {
                sprites.push(
                    self.tile_sheet
                        .create_sprite_for("RESOURCE_GENERIC", real_x, real_y),
                );
            }
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

                            sprites.push(self.tile_sheet.create_sprite_for(class, real_x, real_y));
                            break;
                        }
                    }
                }
            }

            self.tile_sheet_batch.clear();
            self.tile_sheet_batch.extend(sprites);
            self.tile_sheet_batch.draw(&mut frame.as_target());
        }
    }

    fn update(&mut self, window: &Window) -> Option<MainMessage> {
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
                ZoneEventType::ThereIsAround { items } => {
                    self.around_text = items;
                    self.update_link_button_data();
                }
                ZoneEventType::NewResumeText { resume } => {
                    self.resume_text = resume;
                    self.update_link_button_data();
                }
                ZoneEventType::NewBuild { build } => {
                    self.builds.insert(build.id, build);
                    self.update_builds_data();
                }
                _ => println!("unknown event type {:?}", &event.event_type),
            }
        }

        None
    }

    fn interact(&mut self, input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        let mut try_player_moves: Vec<(i16, i16)> = vec![];
        self.cursor_position = input.cursor_position.clone();

        if input.mouse_buttons_pressed.contains(&mouse::Button::Left) {
            let click_x = input.cursor_position.x.round() as i16;
            let click_y = input.cursor_position.y.round() as i16;
            let (to_row_i, to_col_i) = self.xy_to_zone_coords(click_x, click_y);
            input.mouse_buttons_pressed.clear();

            // Is that a move/requested click ? Must not be in menu
            if click_x > START_SCREEN_X && click_y > START_SCREEN_Y {
                if let Some(request_clicks) = &self.request_clicks {
                    // REQUESTED CLICK
                    self.socket.send(event::ZoneEvent {
                        event_type_name: String::from(event::CLICK_ACTION_EVENT),
                        event_type: event::ZoneEventType::ClickActionEvent {
                            base_url: request_clicks.base_url.clone(),
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
            if input.keys_pressed.contains(&keyboard::KeyCode::Right) {
                try_player_moves.push((1, 0));
                self.move_requested = None;
                self.player_tile_id = String::from("PLAYER");
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Left) {
                try_player_moves.push((-1, 0));
                self.move_requested = None;
                self.player_tile_id = String::from("PLAYER_LEFT");
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Up) {
                try_player_moves.push((0, -1));
                self.move_requested = None;
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Down) {
                try_player_moves.push((0, 1));
                self.move_requested = None;
            }
        }

        if let Some(move_requested) = self.move_requested.as_ref() {
            if let Some(next_move) = move_requested.iter().next() {
                try_player_moves.push(*next_move);
                self.move_requested.as_mut().unwrap().remove(0);
            } else {
                self.move_requested = None;
            }
        }

        match input.key_code {
            Some(keyboard::KeyCode::Escape) => {
                input.key_code = None;

                if self.request_clicks.is_some() {
                    self.request_clicks = None;
                } else {
                    return Some(MainMessage::ToExit);
                }
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
        if !player_have_move.0 {
            self.move_requested = None;
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
            _ => {}
        }
        None
    }

    fn layout(&mut self, window: &Window) -> Element {
        let event_class = if self.player.unread_event && self.menu_blinker.visible(500, 'E') {
            thin_button::Class::Primary
        } else {
            thin_button::Class::Secondary
        };
        let business_class =
            if self.player.unread_transactions && self.menu_blinker.visible(500, 'B') {
                thin_button::Class::Primary
            } else {
                thin_button::Class::Secondary
            };
        let affinity_class =
            if self.player.unvote_affinity_relation && self.menu_blinker.visible(500, 'A') {
                thin_button::Class::Primary
            } else {
                thin_button::Class::Secondary
            };
        let zone_message_class =
            if self.player.unread_zone_message && self.menu_blinker.visible(500, 'M') {
                thin_button::Class::Primary
            } else {
                thin_button::Class::Secondary
            };
        let conversation_class =
            if self.player.unread_conversation && self.menu_blinker.visible(500, 'C') {
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
            .width((START_SCREEN_X / 2) as u32)
            .height(window.height() as u32)
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
                    .class(zone_message_class)
                    .on_press(Message::ZoneMenuButtonPressed)
                    .width(175),
            )
            .push(
                Button::new(&mut self.zone_messages_menu_button_state, "Chat")
                    .class(thin_button::Class::Secondary)
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
                Button::new(&mut self.exit_menu_button_state, "Quitter")
                    .class(thin_button::Class::Secondary)
                    .on_press(Message::ExitMenuButtonPressed)
                    .width(175),
            );

        let mut right_menu = Column::new()
            .width((START_SCREEN_X / 2) as u32)
            .height(window.height() as u32);
        for (text, url) in self.resume_text.iter() {
            if let Some(url) = url {
                let id = self.link_button_ids.get(text).unwrap().clone();
                right_menu = right_menu.push(
                    StateLessButton::new(
                        self.link_button_pressed == id,
                        &text,
                        Message::LinkButtonPressed(id),
                        Message::LinkButtonReleased(url.clone()),
                    )
                    .width(175)
                    .class(state_less_button::Class::Positive),
                );
            } else {
                right_menu = right_menu.push(Text::new(text));
            }
        }
        for (text, url) in self.around_text.iter() {
            if let Some(url) = url {
                let id = self.link_button_ids.get(text).unwrap().clone();
                right_menu = right_menu.push(
                    StateLessButton::new(
                        self.link_button_pressed == id,
                        &text,
                        Message::LinkButtonPressed(id),
                        Message::LinkButtonReleased(url.clone()),
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
