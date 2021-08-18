use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use crate::ui::Element;
use coffee::graphics::{Frame, Image, Window};
use coffee::Timer;

pub mod description;
pub mod exit;
pub mod login;
pub mod upgrade;
pub mod world;
pub mod zone;

pub trait Engine {
    fn draw(&mut self, frame: &mut Frame, timer: &Timer, illustration: Option<Image>);
    fn update(&mut self, window: &Window) -> Option<MainMessage>;
    fn interact(&mut self, input: &mut MyGameInput, window: &mut Window) -> Option<MainMessage>;
    fn react(&mut self, event: Message, window: &mut Window) -> Option<MainMessage>;
    fn layout(&mut self, window: &Window, illustration: Option<Image>) -> Element;
    fn teardown(&mut self);
}
