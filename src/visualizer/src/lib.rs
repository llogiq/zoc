// See LICENSE file for copyright and license details.

#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin as gfx_glutin;
extern crate gfx_device_gl as gfx_gl;
extern crate rand;
extern crate time;
extern crate cgmath;
extern crate collision;
extern crate glutin;
extern crate core;
extern crate image;
extern crate rusttype;

mod gui;
mod obj;
mod scene;
mod event_visualizer;
mod unit_type_visual_info;
mod selection;
mod types;
mod pipeline;
mod map_text;
mod move_helper;
mod camera;
mod geom;
mod screen;
mod texture;
mod tactical_screen;
mod context_menu_popup;
mod main_menu_screen;
mod end_turn_screen;
mod context;
mod text;
mod mesh;
mod fs;

use std::sync::mpsc::{channel, Receiver};
use screen::{Screen, ScreenCommand, EventStatus};
use context::{Context};
use main_menu_screen::{MainMenuScreen};
use types::{Time};

pub struct Visualizer {
    screens: Vec<Box<Screen>>,
    popups: Vec<Box<Screen>>,
    should_close: bool,
    last_time: Time,
    context: Context,
    rx: Receiver<ScreenCommand>,
}

impl Visualizer {
    pub fn new() -> Visualizer {
        let (tx, rx) = channel();
        let mut context = Context::new(tx);
        let screens = vec![
            Box::new(MainMenuScreen::new(&mut context)) as Box<Screen>,
        ];
        Visualizer {
            screens: screens,
            popups: Vec::new(),
            should_close: false,
            last_time: Time{n: time::precise_time_ns()},
            context: context,
            rx: rx,
        }
    }

    pub fn tick(&mut self) {
        self.draw();
        self.handle_events();
        self.handle_commands();
    }

    fn draw(&mut self) {
        let dtime = self.update_time();
        self.context.clear_color = [0.8, 0.8, 0.8, 1.0];
        self.context.encoder.clear(&self.context.data.out, self.context.clear_color);
        self.context.encoder.clear_depth(&self.context.data.out_depth, 1.0);
        {
            let screen = self.screens.last_mut().unwrap();
            screen.tick(&mut self.context, &dtime);
        }
        for popup in &mut self.popups {
            popup.tick(&mut self.context, &dtime);
        }
        self.context.encoder.flush(&mut self.context.device);
        self.context.window.swap_buffers()
            .expect("Can`t swap buffers");
    }

    fn handle_events(&mut self) {
        let events: Vec<_> = self.context.window.poll_events().collect();
        for event in &events {
            self.context.handle_event_pre(event);
            let mut event_status = EventStatus::NotHandled;
            for i in (0 .. self.popups.len()).rev() {
                event_status = self.popups[i].handle_event(
                    &mut self.context, event);
                if let EventStatus::Handled = event_status {
                    break;
                }
            }
            if let EventStatus::NotHandled = event_status {
                let screen = self.screens.last_mut().unwrap();
                screen.handle_event(&mut self.context, event);
            }
            self.context.handle_event_post(event);
        }
    }

    fn handle_commands(&mut self) {
        while let Ok(command) = self.rx.try_recv() {
            match command {
                ScreenCommand::PushScreen(screen) => {
                    self.screens.push(screen);
                },
                ScreenCommand::PushPopup(popup) => {
                    self.popups.push(popup);
                },
                ScreenCommand::PopScreen => {
                    self.screens.pop().unwrap();
                    if self.screens.is_empty() {
                        self.should_close = true;
                    }
                    self.popups.clear();
                },
                ScreenCommand::PopPopup => {
                    assert!(self.popups.len() > 0);
                    let _ = self.popups.pop();
                },
            }
        }
    }

    pub fn is_running(&self) -> bool {
        !self.should_close && !self.context.should_close()
    }

    fn update_time(&mut self) -> Time {
        let time = Time{n: time::precise_time_ns()};
        let dtime = Time{n: time.n - self.last_time.n};
        self.last_time = time;
        dtime
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
