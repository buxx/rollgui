use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use coffee::graphics::{Frame, Window};
use coffee::ui::Element;
use coffee::Timer;

pub mod description;
pub mod exit;
pub mod startup;
pub mod zone;

pub trait Engine {
    fn draw(&mut self, frame: &mut Frame, timer: &Timer);
    fn update(&mut self, window: &Window) -> Option<MainMessage>;
    fn interact(&mut self, input: &mut MyGameInput, window: &mut Window) -> Option<MainMessage>;
    fn react(&mut self, event: Message, window: &mut Window) -> Option<MainMessage>;
    fn layout(&mut self, window: &Window) -> Element<Message>;
    fn teardown(&mut self);
}
