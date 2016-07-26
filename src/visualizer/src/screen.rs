// See LICENSE file for copyright and license details.

use glutin::{Event};
use context::{Context};

pub enum ScreenCommand {
    PopScreen,
    PopPopup,
    PushScreen(Box<Screen>),
    PushPopup(Box<Screen>),
}

pub enum EventStatus {
    Handled,
    NotHandled,
}

pub trait Screen {
    fn tick(&mut self, context: &mut Context, dtime: u64);
    fn handle_event(&mut self, context: &mut Context, event: &Event) -> EventStatus;
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
