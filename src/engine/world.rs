use crate::engine::Engine;
use crate::entity::player::Player;
use crate::game::{TILE_HEIGHT, TILE_WIDTH};
use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use crate::server::Server;
use crate::sheet::TileSheet;
use crate::ui::Column;
use crate::ui::Element;
use crate::util::Blinker;
use coffee::graphics::{
    Batch, Color, Frame, HorizontalAlignment, Sprite, VerticalAlignment, Window,
};
use coffee::input::keyboard;
use coffee::ui::Text;
use coffee::{graphics, Timer};
use std::collections::HashMap;

pub struct WorldEngine {
    server: Server,
    tile_sheet: TileSheet,
    tile_sheet_batch: Batch,
    player: Player,
    blinker: Blinker<char>,
    start_screen_x: i16,
    start_screen_y: i16,
    end_screen_x: i16,
    end_screen_y: i16,
    start_world_row_i: i16,
    start_world_col_i: i16,
    move_x: f32,
    move_y: f32,
}

impl WorldEngine {
    pub fn new(server: Server, tile_sheet_image: graphics::Image, player: Player) -> Self {
        Self {
            server,
            tile_sheet: TileSheet::new(tile_sheet_image.clone(), TILE_WIDTH, TILE_HEIGHT),
            tile_sheet_batch: Batch::new(tile_sheet_image.clone()),
            player,
            blinker: Blinker {
                items: HashMap::new(),
            },
            start_screen_x: 0,
            start_screen_y: 0,
            end_screen_x: -1,
            end_screen_y: -1,
            start_world_row_i: 0,
            start_world_col_i: 0,
            move_x: 0.0,
            move_y: 0.0,
        }
    }

    fn real_x(&self, x: i16) -> i16 {
        (self.tile_sheet.get_tile_width() * x) + self.start_screen_x
    }

    fn real_y(&self, y: i16) -> i16 {
        (self.tile_sheet.get_tile_height() * y) + self.start_screen_y
    }

    fn get_world_sprites(&mut self) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = vec![];
        let can_display_rows =
            (self.end_screen_y - self.start_screen_y) / self.tile_sheet.get_tile_height();
        let can_display_cols =
            (self.end_screen_x - self.start_screen_x) / self.tile_sheet.get_tile_width();

        for absolute_row_i in 0..can_display_rows {
            let world_row_i =
                absolute_row_i + (self.start_world_row_i + self.move_y.round() as i16);

            for absolute_col_i in 0..can_display_cols {
                let world_col_i =
                    absolute_col_i + (self.start_world_col_i + self.move_x.round() as i16);

                if world_row_i >= self.server.world.rows.len() as i16 || world_row_i < 0 {
                    sprites.push(self.tile_sheet.create_sprite_for(
                        "SEA",
                        self.real_x(absolute_col_i),
                        self.real_y(absolute_row_i),
                    ));
                    continue;
                }

                let row = &self.server.world.rows[world_row_i as usize];

                if world_col_i >= row.cols.len() as i16 || world_col_i < 0 {
                    sprites.push(self.tile_sheet.create_sprite_for(
                        "SEA",
                        self.real_x(absolute_col_i),
                        self.real_y(absolute_row_i),
                    ));
                    continue;
                }

                let tile_type_id = row.cols[world_col_i as usize].clone();
                sprites.push(self.tile_sheet.create_sprite_for(
                    &tile_type_id,
                    self.real_x(absolute_col_i),
                    self.real_y(absolute_row_i),
                ));

                if world_row_i == self.player.world_position.0 as i16
                    && world_col_i == self.player.world_position.1 as i16
                {
                    if self.blinker.visible(500, 'C') {
                        sprites.push(self.tile_sheet.create_sprite_for(
                            "PLAYER",
                            self.real_x(absolute_col_i),
                            self.real_y(absolute_row_i),
                        ));
                    }
                }
            }
        }

        sprites
    }
}

impl Engine for WorldEngine {
    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        frame.clear(Color::BLACK);

        if timer.has_ticked() {
            let mut sprites: Vec<Sprite> = vec![];

            sprites.extend(self.get_world_sprites());

            self.tile_sheet_batch.clear();
            self.tile_sheet_batch.extend(sprites);
            self.tile_sheet_batch.draw(&mut frame.as_target());
        }
    }

    fn update(&mut self, window: &Window) -> Option<MainMessage> {
        self.end_screen_y = window.height() as i16;
        self.end_screen_x = window.width() as i16;

        self.start_world_row_i = -(window.height().round() as i16 / TILE_HEIGHT) / 2;
        self.start_world_col_i = -(window.width().round() as i16 / TILE_WIDTH) / 2;

        None
    }

    fn interact(&mut self, input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        if !input.keys_pressed.is_empty() {
            if input.keys_pressed.contains(&keyboard::KeyCode::Right) {
                self.move_x += 0.1;
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Left) {
                self.move_x -= 0.1;
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Up) {
                self.move_y -= 0.1;
            }
            if input.keys_pressed.contains(&keyboard::KeyCode::Down) {
                self.move_y += 0.1;
            }
        }

        if input.key_code == Some(keyboard::KeyCode::Escape) {
            return Some(MainMessage::DescriptionToZone);
        }

        None
    }

    fn react(&mut self, _event: Message, _window: &mut Window) -> Option<MainMessage> {
        None
    }

    fn layout(&mut self, window: &Window) -> Element<Message> {
        Column::new()
            .width(window.width() as u32)
            .height(window.height() as u32)
            .push(
                Text::new("Echap: retour")
                    .horizontal_alignment(HorizontalAlignment::Right)
                    .vertical_alignment(VerticalAlignment::Top),
            )
            .into()
    }

    fn teardown(&mut self) {}
}
