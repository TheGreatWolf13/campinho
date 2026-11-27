mod arena;
mod atlas;
pub mod colour;
pub mod font;
pub mod gl;
mod util;

use crate::arena::Arena;
use crate::colour::ColourWheelManager;
use crate::font::FontSystem;
use crate::gl::frame::Frame;
use crate::gl::shader::{Shader, ShaderManager, UniformProjViewThickness};
use crate::gl::vertex_buffer::{VertexBorder, VertexPosColour};
use cgmath::{EuclideanSpace, Point2};
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::winit::application::ApplicationHandler;
use glium::winit::event::{KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use glium::winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::window::{Icon, Window, WindowAttributes, WindowId};
use glium::Display;
use log::{debug, info};
use std::time::{Duration, Instant};

struct Campinho {
    window: Window,
    arena: Arena,
    shader_manager: ShaderManager,
    display: Display<WindowSurface>,
    fps: usize,
    last_fps_time: Instant,
    last_frame_time: Instant,
    accumulated_frame_time: Duration,
    font_system: FontSystem,
    draw_calls: u32,
    middle_button_held: bool,
    last_cursor_pos: Point2<f32>,
    colour_wheel_manager: ColourWheelManager,
}

impl ApplicationHandler for Campinho {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll)
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.window.request_redraw();
                let current_time = Instant::now();
                self.arena.update(current_time - self.last_frame_time);
                self.last_frame_time = current_time;
                self.render();
                self.accumulated_frame_time += Instant::now() - current_time;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => self.scroll(y),
                    MouseScrollDelta::PixelDelta(p) => self.scroll(p.y as f32),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position = Point2::new(position.x as f32, position.y as f32);
                let delta = position - self.last_cursor_pos;
                self.last_cursor_pos = position;
                self.arena.mouse_moved(delta, self.middle_button_held, self.window.inner_size());
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match button {
                    MouseButton::Left => {}
                    MouseButton::Right => {}
                    MouseButton::Middle => {
                        if state.is_pressed() {
                            self.middle_button_held = true;
                        } //
                        else {
                            self.middle_button_held = false;
                        }
                    }
                    MouseButton::Back => {}
                    MouseButton::Forward => {}
                    MouseButton::Other(_) => {}
                }
            }
            _ => (),
        }
    }
}

impl Campinho {
    fn new(window: Window, display: Display<WindowSurface>) -> Self {
        Self {
            shader_manager: ShaderManager::new(&display),
            arena: Arena::new(&display),
            font_system: FontSystem::new(&display),
            window,
            fps: 0,
            last_fps_time: Instant::now(),
            last_frame_time: Instant::now(),
            accumulated_frame_time: Duration::ZERO,
            draw_calls: 0,
            middle_button_held: false,
            last_cursor_pos: Point2::origin(),
            colour_wheel_manager: ColourWheelManager::new(&display),
            display,
        }
    }

    fn scroll(&mut self, delta: f32) {
        self.arena.zoom(delta);
    }

    fn handle_key(&mut self, event: KeyEvent) {
        match event.physical_key {
            PhysicalKey::Code(k) => {
                match k {
                    KeyCode::KeyW | KeyCode::KeyS | KeyCode::KeyA | KeyCode::KeyD => self.arena.handle_keys(k, event.state.is_pressed()),
                    KeyCode::F4 => {
                        if event.state.is_pressed() {
                            self.arena.toggle_capture_frustum();
                        }
                    }
                    _ => (),
                }
            }
            _ => {}
        }
    }

    fn render(&mut self) {
        let current_time = Instant::now();
        if (current_time - self.last_fps_time).as_secs_f32() > 1.0 {
            self.last_fps_time = current_time;
            debug!("FPS: {} / Average Frame Time: {:1.2} ms / Average draw calls per frame: {}", self.fps, self.accumulated_frame_time.as_secs_f64() * 1_000.0 / self.fps as f64, self.draw_calls as f64 / self.fps as f64);
            self.fps = 0;
            self.accumulated_frame_time = Duration::ZERO;
            self.draw_calls = 0;
        }
        let mut frame = Frame::new(&self.display);
        let size = self.window.inner_size();
        self.arena.render(&mut frame, &self.shader_manager, &mut self.font_system, size);
        self.draw_calls += frame.draw_calls();
        self.fps += 1;
    }
}

fn main() {
    unsafe {
        //SAFETY: called from a single threaded environment
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::builder().format_source_path(true).format_target(false).init();
    let event_loop = EventLoop::builder().build().expect("Failed to create event loop");
    let (window, display) = SimpleWindowBuilder::new()
        .set_window_builder(WindowAttributes::default()
            .with_window_icon({
                match image::open("res/icon.ico").map(|i| i.into_rgba8()).ok() {
                    None => None,
                    Some(img) => {
                        let (width, height) = img.dimensions();
                        Icon::from_rgba(img.into_raw(), width, height).ok()
                    }
                }
            })
        )
        .with_title("Campinho Eletrônico V0.1.0")
        .build(&event_loop);
    let mut campinho = Campinho::new(window, display);
    event_loop.run_app(&mut campinho).unwrap();
}
