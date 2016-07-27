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

// TODO: убрать в честный модуль
pub mod types {
    use gfx;
    use cgmath::{Vector3, Vector2};

    pub use core::types::{ZInt, ZFloat, Size2};

    // TODO: вынести куда-нибудь, это же вообще не типы
    pub type ColorFormat = gfx::format::Srgba8;
    pub type DepthFormat = gfx::format::DepthStencil;
    pub type SurfaceFormat = gfx::format::R8_G8_B8_A8;
    pub type FullFormat = (SurfaceFormat, gfx::format::Unorm);

    // это надо вынести в какой-то отдельный модуль
    #[derive(Copy, Clone)]
    pub struct WorldPos{pub v: Vector3<ZFloat>}

    // его надо убить
    #[derive(Copy, Clone)]
    pub struct VertexCoord{pub v: Vector3<ZFloat>}

    // его надо убить
    #[derive(Copy, Clone)]
    pub struct TextureCoord{pub v: Vector2<ZFloat>}

    #[derive(Copy, Clone)]
    pub struct ScreenPos{pub v: Vector2<ZInt>}

    #[derive(Copy, Clone)]
    pub struct Time{pub n: u64}

    #[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
    pub struct MeshId{pub id: ZInt}
}

use std::sync::mpsc::{channel, Receiver};
use screen::{Screen, ScreenCommand, EventStatus};
use context::{Context};
use main_menu_screen::{MainMenuScreen};

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline pipe {
        basic_color: gfx::Global<[f32; 4]> = "u_Basic_color",
        mvp: gfx::Global<[[f32; 4]; 4]> = "u_ModelViewProj",
        vbuf: gfx::VertexBuffer<Vertex> = (),
        texture: gfx::TextureSampler<[f32; 4]> = "t_Tex",
        out: gfx::BlendTarget<types::ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        out_depth: gfx::DepthTarget<types::DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub struct Visualizer {
    screens: Vec<Box<Screen>>,
    popups: Vec<Box<Screen>>,
    should_close: bool, // объединит с такой же фигней в контексте =\
    last_time: u64,
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
            last_time: time::precise_time_ns(),
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
            screen.tick(&mut self.context, dtime);
        }
        for popup in &mut self.popups {
            popup.tick(&mut self.context, dtime);
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

    fn update_time(&mut self) -> u64 {
        let time = time::precise_time_ns();
        let dtime = time - self.last_time;
        self.last_time = time;
        dtime
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
