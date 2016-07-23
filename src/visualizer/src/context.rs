// See LICENSE file for copyright and license details.

use std::sync::mpsc::{Sender};
// use std::path::{Path};
use cgmath::{Vector2, Array};
use glutin::{self, Api, Event, MouseButton, GlRequest};
use glutin::ElementState::{Pressed, Released};
use core::types::{Size2, ZInt};
use screen::{ScreenCommand};
use types::{ScreenPos, /*Color4,*/ ColorFormat, SurfaceFormat};
use ::{pipe};

use image;
use std::io::Cursor;
use gfx::traits::{Factory, FactoryExt};
use gfx::handle::{RenderTargetView, DepthStencilView, ShaderResourceView};
use gfx::{self, tex};
use gfx_gl;
use gfx_glutin;
use core::fs;

// TODO: найти более подходящее место
pub fn load_texture<R, F>(factory: &mut F, data: &[u8]) -> ShaderResourceView<R, [f32; 4]>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = tex::Kind::D2(width as tex::Size, height as tex::Size, tex::AaMode::Single);
    let (_, view) = factory.create_texture_const_u8::<ColorFormat>(kind, &[&img]).unwrap();
    view
}

fn new_pso(
    window: &glutin::Window,
    factory: &mut gfx_gl::Factory,
    primitive: gfx::Primitive,
) -> gfx::PipelineState<gfx_gl::Resources, pipe::Meta> {
    let shader_header = match window.get_api() {
        Api::OpenGl => fs::load("shader/pre_gl.glsl").into_inner(),
        Api::OpenGlEs | Api::WebGl => fs::load("shader/pre_gles.glsl").into_inner(),
    };
    let mut vertex_shader = shader_header.clone();
    vertex_shader.extend(fs::load("shader/v.glsl").into_inner());
    let mut fragment_shader = shader_header;
    fragment_shader.extend(fs::load("shader/f.glsl").into_inner());
    let vs = factory.create_shader_vertex(&vertex_shader).unwrap();
    let ps = factory.create_shader_pixel(&fragment_shader).unwrap();
    let shader_set = gfx::ShaderSet::Simple(vs, ps);
    factory.create_pipeline_state(
        &shader_set,
        primitive,
        gfx::state::Rasterizer::new_fill(),
        pipe::new(),
    ).unwrap()
}

fn get_win_size(window: &glutin::Window) -> Size2 {
    let (w, h) = window.get_inner_size().expect("Can`t get window size");
    Size2{w: w as ZInt, h: h as ZInt}
}

pub struct MouseState {
    pub is_left_button_pressed: bool,
    pub is_right_button_pressed: bool,
    pub last_press_pos: ScreenPos,
    pub pos: ScreenPos,
}

// TODO: make more fields private?
pub struct Context {
    pub win_size: Size2,
    // pub font_stash: FontStash,
    mouse: MouseState,
    should_close: bool,
    commands_tx: Sender<ScreenCommand>,

    // ------

    // TODO: все публичное, да?
    pub window: glutin::Window,
    pub clear_color: [f32; 4],
    pub device: gfx_gl::Device,
    pub main_color: RenderTargetView<gfx_gl::Resources, (SurfaceFormat, gfx::format::Srgb)>,
    pub main_depth: DepthStencilView<gfx_gl::Resources, (gfx::format::D24_S8, gfx::format::Unorm)>,
    pub encoder: gfx::Encoder<gfx_gl::Resources, gfx_gl::CommandBuffer>,
    pub pso: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    pub pso_wire: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    pub sampler: gfx::handle::Sampler<gfx_gl::Resources>,
    pub factory: gfx_gl::Factory,
}

impl Context {
    pub fn new(tx: Sender<ScreenCommand>) -> Context {

        let gl_version = GlRequest::GlThenGles {
            opengles_version: (2, 0),
            opengl_version: (2, 1),
        };
        let builder = glutin::WindowBuilder::new()
            .with_title("Zone of Control".to_string())
            .with_pixel_format(24, 8)
            .with_gl(gl_version);
        let (window, device, mut factory, main_color, main_depth)
            = gfx_glutin::init(builder);
        let encoder = factory.create_command_buffer().into();
        let pso = new_pso(&window, &mut factory, gfx::Primitive::TriangleList);
        let pso_wire = new_pso(&window, &mut factory, gfx::Primitive::LineList);
        let sampler = factory.create_sampler_linear();
        let win_size = get_win_size(&window);
        // let font_size = 40.0;
        // // TODO: read font name from config
        // let font_stash = FontStash::new(
        //     &zgl, &Path::new("DroidSerif-Regular.ttf"), font_size);
        Context {
            // window: window,
            win_size: win_size,
            // font_stash: font_stash,

            clear_color: [0.0, 0.0, 1.0, 1.0],
            window: window,
            device: device,
            factory: factory,
            main_color: main_color,
            main_depth: main_depth,
            encoder: encoder,
            pso: pso,
            pso_wire: pso_wire,
            sampler: sampler,
            should_close: false,
            commands_tx: tx,
            mouse: MouseState {
                is_left_button_pressed: false,
                is_right_button_pressed: false,
                last_press_pos: ScreenPos{v: Vector2::from_value(0)},
                pos: ScreenPos{v: Vector2::from_value(0)},
            },
        }
    }

    pub fn should_close(&self) -> bool {
        self.should_close
    }

    pub fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    /*
    // если data живет вовне (что может быть ошибкой, кстати) то и этот метод должен быть не тут
    pub fn set_basic_color(&mut self, color: &Color4) {
        self.data.basic_color = [color.r, color.g, color.b, color.r];
    }
    */

    pub fn add_command(&mut self, command: ScreenCommand) {
        self.commands_tx.send(command)
            .expect("Can't send command to Visualizer");
    }

    pub fn handle_event_pre(&mut self, event: &glutin::Event) {
        match *event {
            Event::Closed => {
                self.should_close = true;
            },
            Event::MouseInput(Pressed, MouseButton::Left) => {
                self.mouse.is_left_button_pressed = true;
                self.mouse.last_press_pos = self.mouse.pos.clone();
            },
            Event::MouseInput(Released, MouseButton::Left) => {
                self.mouse.is_left_button_pressed = false;
            },
            Event::MouseInput(Pressed, MouseButton::Right) => {
                self.mouse.is_right_button_pressed = true;
            },
            Event::MouseInput(Released, MouseButton::Right) => {
                self.mouse.is_right_button_pressed = false;
            },
            Event::Resized(w, h) => {
                self.win_size = Size2{w: w as ZInt, h: h as ZInt};
                // self.zgl.set_viewport(&self.win_size);
            },
            _ => {},
        }
    }

    pub fn handle_event_post(&mut self, event: &glutin::Event) {
        match *event {
            Event::MouseMoved(x, y) => {
                let pos = ScreenPos{v: Vector2{x: x as ZInt, y: y as ZInt}};
                self.mouse.pos = pos;
            },
            Event::Touch(glutin::Touch{location: (x, y), phase, ..}) => {
                let pos = ScreenPos{v: Vector2{x: x as ZInt, y: y as ZInt}};
                match phase {
                    glutin::TouchPhase::Moved => {
                        self.mouse.pos = pos;
                    },
                    glutin::TouchPhase::Started => {
                        self.mouse.pos = pos.clone();
                        self.mouse.last_press_pos = pos;
                        self.mouse.is_left_button_pressed = true;
                    },
                    glutin::TouchPhase::Ended => {
                        self.mouse.pos = pos;
                        self.mouse.is_left_button_pressed = false;
                    },
                    glutin::TouchPhase::Cancelled => {
                        unimplemented!();
                    },
                }
            },
            _ => {},
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
