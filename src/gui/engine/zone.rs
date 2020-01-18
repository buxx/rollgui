use doryen_rs::{DoryenApi, TextAlign, UpdateEvent};
use std::collections::HashMap;

use crate::color;
use crate::entity::character::Character;
use crate::entity::player::Player;
use crate::event;
use crate::event::ZoneEventType;
use crate::gui::engine::Engine;
use crate::server;
use crate::tile::Tiles;
use crate::zone::level::Level;
use crate::zone::socket::ZoneSocket;
use std::any::Any;

pub struct ZoneEngine {
    _server: server::Server,
    player: Player,
    characters: HashMap<String, Character>,
    socket: ZoneSocket,
    level: Level,
    tiles: Tiles,
    // Map coordinates where start to display it
    start_display_map_row_i: i32,
    start_display_map_col_i: i32,
    mouse_pos: (f32, f32),
}

impl ZoneEngine {
    pub fn new(
        server: server::Server,
        player: Player,
        characters: HashMap<String, Character>,
        socket: ZoneSocket,
        level: Level,
        tiles: Tiles,
        start_display_map_row_i: i32,
        start_display_map_col_i: i32,
    ) -> Self {
        Self {
            _server: server,
            player,
            characters,
            socket,
            level,
            tiles,
            start_display_map_row_i,
            start_display_map_col_i,
            mouse_pos: (0.0, 0.0),
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

    fn update(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) -> Option<UpdateEvent> {
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
                        eprintln!("Unknown character {}", character_id)
                    }
                }
                _ => {}
            }
        }

        let mut mov = self.player.move_from_input(api);
        let mut coef = 1.0 / std::f32::consts::SQRT_2;

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
        );
        self.player.render(api, width as i32, height as i32);

        let con = api.con();
        for (character_id, character) in self.characters.iter() {
            con.ascii(
                character.zone_col_i - self.start_display_map_col_i,
                character.zone_row_i - self.start_display_map_row_i,
                '%' as u16,
            );
            con.fore(
                character.zone_col_i - self.start_display_map_col_i,
                character.zone_row_i - self.start_display_map_row_i,
                color::WHITE,
            );
        }

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
}
