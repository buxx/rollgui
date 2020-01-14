use doryen_rs::{DoryenApi, UpdateEvent, TextAlign};

use crate::entity::player::{Player};
use crate::zone::level::{Level};
use crate::tile::{Tiles};
use crate::gui::engine::{Engine};
use crate::zone::socket::{ZoneSocket};
use crate::event;


pub struct ZoneEngine {
    player: Player,
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
        player: Player,
        socket: ZoneSocket,
        level: Level,
        tiles: Tiles,
        start_display_map_row_i: i32,
        start_display_map_col_i: i32,
    ) -> Self {
        Self {
            player,
            socket,
            level,
            tiles,
            start_display_map_row_i,
            start_display_map_col_i,
            mouse_pos: (0.0, 0.0),
        }
    }

    pub fn can_move(&self, position: (i32, i32)) -> bool {
        let tile = self.level.tile(position);
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
            println!("Server to gui: {}", event.event_type_name);
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
            self.socket.send(event::ZoneEvent{
                event_type_name: String::from(event::PLAYER_MOVE),
                event_type: event::ZoneEventType::PlayerMove{
                    to_row_i: self.player.position.0 as i32,
                    to_col_i: self.player.position.1 as i32,
                    character_id: "abc".to_string(),
                },
            });
        }

        self.start_display_map_row_i = self.player.position.0 as i32 - (height / 2);
        self.start_display_map_col_i = self.player.position.1 as i32 - (width / 2);

        self.mouse_pos = api.input().mouse_pos();
        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi, width: i32, height: i32) {
        self.level.render(api, &self.tiles, self.start_display_map_row_i, self.start_display_map_col_i, width, height);
        self.player.render(api, width as i32, height as i32);

        let fps = api.fps();
        api.con().print_color(
            1,
            20,
            &format!("row {} / col {} {}fps", self.mouse_pos.1 as i32, self.mouse_pos.0 as i32, fps),
            TextAlign::Left,
            None,
        );
        api.con().back(self.mouse_pos.0 as i32, self.mouse_pos.1 as i32, (255, 255, 255, 255));
    }
    fn resize(&mut self, _api: &mut dyn DoryenApi) {}

    fn teardown(&mut self) {
        // TODO: manage case where fail to close
        self.socket.close().unwrap();
    }
}