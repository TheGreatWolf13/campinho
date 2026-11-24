use crate::colour::{ColourHex, ColourU8};
use crate::font::{FontSystem, GlyphVertex, HorizontalAlignment, VerticalAlignment};
use crate::gl::shader::{Shader, UniformProjView, UniformProjViewThickness};
use crate::gl::vertex_buffer::{BitSet3, VertexBorder, VertexBuffer};
use crate::{if_else, Frame, VertexPosColour};
use ab_glyph::{point, Rect};
use cgmath::{ortho, Matrix4, Point3, Vector2, Vector3};
use glium::glutin::surface::WindowSurface;
use glium::index::NoIndices;
use glium::index::PrimitiveType::{LinesList, TrianglesList};
use glium::winit::dpi::PhysicalSize;
use glium::winit::keyboard::KeyCode;
use glium::{Blend, Display, DrawParameters};
use std::time::Duration;

pub(super) struct Arena {
    zoom_level: f32,
    vx: f32,
    vy: f32,
    x: f32,
    y: f32,
    forward: bool,
    backward: bool,
    leftward: bool,
    rightward: bool,
    grid_lines_vertex_buffer: VertexBuffer<VertexPosColour>,
    grid_numbers_vertex_buffer: VertexBuffer<GlyphVertex>,
    frustum: FrustumHolder,
    zoom_timer: Duration,
}

#[derive(Copy, Clone)]
struct Frustum(Rect);

#[derive(Copy, Clone)]
enum FrustumHolder {
    None,
    ShouldCapture,
    Captured(Frustum),
}

impl FrustumHolder {
    fn unwrap_or_else(self, f: impl FnOnce() -> Frustum) -> Frustum {
        match self {
            FrustumHolder::None | FrustumHolder::ShouldCapture => f(),
            FrustumHolder::Captured(frustum) => frustum,
        }
    }
}

impl Frustum {
    fn new(x: f32, y: f32, half_dx: f32, half_dy: f32) -> Self {
        Self(Rect {
            min: point(x - half_dx, y - half_dy),
            max: point(x + half_dx, y + half_dy),
        })
    }

    fn get_x0(&self) -> f32 {
        self.0.min.x
    }

    fn get_y0(&self) -> f32 {
        self.0.min.y
    }

    fn get_x1(&self) -> f32 {
        self.0.max.x
    }

    fn get_y1(&self) -> f32 {
        self.0.max.y
    }
}

impl Arena {
    pub(super) fn new(display: &Display<WindowSurface>) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            zoom_level: 1.0,
            forward: false,
            backward: false,
            leftward: false,
            rightward: false,
            grid_lines_vertex_buffer: VertexBuffer::new("Grid Lines Vertex Buffer", display, 64),
            grid_numbers_vertex_buffer: VertexBuffer::new("Grid Number Vertex Buffer", display, 64),
            frustum: FrustumHolder::None,
            zoom_timer: Duration::from_secs(0),
        }
    }

    pub(super) fn toggle_capture_frustum(&mut self) {
        if let FrustumHolder::None = self.frustum {
            self.frustum = FrustumHolder::ShouldCapture;
        } //
        else {
            self.frustum = FrustumHolder::None;
        }
    }

    pub(super) fn mouse_moved(&mut self, delta: Vector2<f32>, middle_pressed: bool, size: PhysicalSize<u32>) {
        if middle_pressed {
            let view_range = 16.0 * self.zoom_level;
            let aspect_ratio = size.width as f32 / size.height as f32;
            let delta_x = delta.x / size.width as f32;
            let delta_y = delta.y / size.height as f32;
            self.x -= delta_x * view_range * aspect_ratio;
            self.y += delta_y * view_range;
        }
    }

    pub(super) fn handle_keys(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.forward = pressed,
            KeyCode::KeyA => self.leftward = pressed,
            KeyCode::KeyS => self.backward = pressed,
            KeyCode::KeyD => self.rightward = pressed,
            _ => (),
        }
    }

    pub(super) fn zoom(&mut self, delta: f32) {
        self.zoom_timer = Duration::from_secs(2);
        if delta < 0.0 {
            self.zoom_level *= 2.0;
            self.zoom_level = self.zoom_level.min(16.0);
        } //
        else if delta > 0.0 {
            self.zoom_level /= 2.0;
            self.zoom_level = self.zoom_level.max(1.0 / 16.0);
        }
    }

    pub(super) fn update(&mut self, dt: Duration) {
        self.vx = 0.0;
        self.vy = 0.0;
        self.vx += if_else!(self.rightward => 1.0 ; 0.0);
        self.vx += if_else!(self.leftward => -1.0 ; 0.0);
        self.vy += if_else!(self.forward => 1.0 ; 0.0);
        self.vy += if_else!(self.backward => -1.0 ; 0.0);
        self.vx *= self.zoom_level * 8.0;
        self.vy *= self.zoom_level * 8.0;
        self.x += self.vx * dt.as_secs_f32();
        self.y += self.vy * dt.as_secs_f32();
        self.zoom_timer = self.zoom_timer.saturating_sub(dt);
    }

    pub(super) fn render(&mut self, frame: &mut Frame, default_shader: &Shader<VertexPosColour, UniformProjView>, border_shader: &Shader<VertexBorder, UniformProjViewThickness>, font: &mut FontSystem, size: PhysicalSize<u32>) {
        let aspect_ratio = size.width as f32 / size.height as f32;
        frame.clear_colour(ColourU8(25, 25, 25, 255));
        self.draw_grid(frame, font, default_shader, aspect_ratio);
        self.draw_ui(frame, font, default_shader, border_shader, aspect_ratio);
    }

    fn draw_ui(&mut self, frame: &mut Frame, font: &mut FontSystem, default_shader: &Shader<VertexPosColour, UniformProjView>, border_shader: &Shader<VertexBorder, UniformProjViewThickness>, aspect_ratio: f32) {
        let display = frame.display();
        let proj_matrix = ortho(
            -aspect_ratio * 50.0,
            aspect_ratio * 50.0,
            -50.0,
            50.0,
            100.0,
            -100.0,
        );
        let view_matrix = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        //Draw zoom
        if self.zoom_timer > Duration::ZERO {
            let zoom_level_text = format!("x{}", self.zoom_level);
            let alpha = if self.zoom_timer > Duration::from_secs(1) {
                255
            } //
            else {
                (self.zoom_timer.as_secs_f32() * 255.0) as u8
            };
            let (text_width, text_height) = font.width_and_height(&zoom_level_text, 10.0, display);
            {
                let mut writer = self.grid_lines_vertex_buffer.write(display);
                let mut stack = writer.stack();
                let alpha = alpha as f32 / 255.0;
                let colour = ColourU8(31, 31, 31, (191.0 * alpha) as u8);
                stack.push_quad([
                    VertexPosColour::new(50.0 * aspect_ratio - 0.5, -50.0 + text_height - 0.5, colour),
                    VertexPosColour::new(50.0 * aspect_ratio - 0.5, -50.0 + 0.5, colour),
                    VertexPosColour::new(50.0 * aspect_ratio - text_width - 1.5, -50.0 + 0.5, colour),
                    VertexPosColour::new(50.0 * aspect_ratio - text_width - 1.5, -50.0 + text_height - 0.5, colour),
                ]);
            }
            let uniforms = UniformProjView::new(proj_matrix, view_matrix);
            let mut draw_parameters = DrawParameters::default();
            draw_parameters.blend = Blend::alpha_blending();
            frame.draw(default_shader, &self.grid_lines_vertex_buffer, NoIndices(TrianglesList), &uniforms, &draw_parameters);
            font.render_text(frame, &mut self.grid_numbers_vertex_buffer, &zoom_level_text, HorizontalAlignment::Right(50.0 * aspect_ratio - 1.0), VerticalAlignment::Bottom(-49.0), 10.0, ColourU8(0, 255, 0, alpha), proj_matrix, view_matrix);
        }
        //Draw char creator
        {
            let mut buffer = VertexBuffer::new("Char buffer", display, 3);
            let fill_colour = ColourHex(0xFFAAAFC3);
            let border_colour = ColourHex(0xFF9398AC);
            {
                let mut writer = buffer.write(display);
                let mut stack = writer.stack();
                stack.push_bordered_triangle(
                    [
                        [10.0, 10.0, 0.0],
                        [-10.0, 0.0, 0.0],
                        [0.0, -10.0, 0.0],
                    ],
                    fill_colour,
                    border_colour,
                    BitSet3::B110,
                );
            }
            let uniforms = UniformProjViewThickness::new(proj_matrix, view_matrix, 10.0);
            let mut draw_parameters = DrawParameters::default();
            draw_parameters.blend = Blend::alpha_blending();
            frame.draw(border_shader, &buffer, NoIndices(TrianglesList), &uniforms, &draw_parameters);
        }
    }

    fn draw_grid(&mut self, frame: &mut Frame, font: &mut FontSystem, shader: &Shader<VertexPosColour, UniformProjView>, aspect_ratio: f32) {
        let view_range = 8.0 * self.zoom_level;
        let proj_matrix = ortho(
            -aspect_ratio * view_range,
            aspect_ratio * view_range,
            -view_range,
            view_range,
            100.0,
            -100.0,
        );
        let eye = Point3::new(self.x, self.y, 100.0);
        let target = Point3::new(self.x, self.y, 0.0);
        let up = Vector3::unit_y();
        let view_matrix = Matrix4::look_at_rh(eye, target, up);
        let uniforms = UniformProjView::new(proj_matrix, view_matrix);
        let frustum = self.frustum.unwrap_or_else(|| Frustum::new(self.x, self.y, view_range * aspect_ratio, view_range));
        if let FrustumHolder::ShouldCapture = self.frustum {
            self.frustum = FrustumHolder::Captured(frustum);
        }
        let mut draw_params = DrawParameters::default();
        draw_params.line_width = Some(3.0 / self.zoom_level);
        let x0 = frustum.get_x0() as i32 - 1;
        let x1 = frustum.get_x1() as i32 + 1;
        let y0 = frustum.get_y0() as i32 - 1;
        let y1 = frustum.get_y1() as i32 + 1;
        let display = frame.display();
        {
            let mut writer = self.grid_lines_vertex_buffer.write(display);
            let mut stack = writer.stack();
            for x in x0..=x1 {
                stack.push(Self::grid_vertex(x as f32, y0 as f32));
                stack.push(Self::grid_vertex(x as f32, y1 as f32));
            }
            for y in y0..=y1 {
                stack.push(Self::grid_vertex(x0 as f32, y as f32));
                stack.push(Self::grid_vertex(x1 as f32, y as f32));
            }
        }
        frame.draw(&shader, &self.grid_lines_vertex_buffer, NoIndices(LinesList), &uniforms, &draw_params);
        if self.zoom_level <= 4.0 {
            let mut batch = font.render_in_batch(frame, ColourU8(18, 245, 255, 255), proj_matrix, view_matrix);
            {
                let mut writer = self.grid_numbers_vertex_buffer.write(display);
                let mut stack = writer.stack();
                for x in x0..x1 {
                    for y in y0..y1 {
                        batch.render_text(&format!("{}", x), HorizontalAlignment::Left(x as f32 + 0.05), VerticalAlignment::Bottom(y as f32 + 0.7), 0.75, &mut stack, display);
                    }
                }
            }
            batch.end_batch(&self.grid_numbers_vertex_buffer);
            let mut batch = font.render_in_batch(frame, ColourU8(224, 166, 245, 255), proj_matrix, view_matrix);
            {
                let mut writer = self.grid_numbers_vertex_buffer.write(display);
                let mut stack = writer.stack();
                for x in (x0 + 1)..(x1 + 1) {
                    for y in y0..y1 {
                        batch.render_text(&format!("{}", y), HorizontalAlignment::Right(x as f32 - 0.05), VerticalAlignment::Bottom(y as f32 + 0.1), 0.75, &mut stack, display);
                    }
                }
            }
            batch.end_batch(&self.grid_numbers_vertex_buffer);
        }
    }

    fn grid_vertex(x: f32, y: f32) -> VertexPosColour {
        VertexPosColour {
            pos: [x, y, 0.0],
            colour: [1.0, 1.0, 1.0, 1.0],
        }
    }
}