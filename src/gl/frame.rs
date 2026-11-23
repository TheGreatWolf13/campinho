use crate::colour::Colour;
use crate::gl::shader::Shader;
use crate::gl::vertex_buffer::VertexBuffer;
use glium::glutin::surface::WindowSurface;
use glium::uniforms::Uniforms;
use glium::{index, Display, DrawParameters, Surface, Vertex};

///<h1>Frame</h1>
pub struct Frame<'display> {
    display: &'display Display<WindowSurface>,
    frame: Option<glium::Frame>,
    draw_calls: u32,
}

impl<'display> Frame<'display> {
    pub fn new(display: &'display Display<WindowSurface>) -> Self {
        Self {
            display,
            frame: Some(display.draw()),
            draw_calls: 0,
        }
    }

    pub fn clear_colour(&mut self, colour: impl Colour) {
        self.frame.as_mut().unwrap().clear_color_srgb(colour.r(), colour.g(), colour.b(), colour.a());
    }

    pub fn draw<'a, V: Vertex, U: Uniforms>(&mut self, shader: &Shader<V, U>, vertex_buffer: &VertexBuffer<V>, index_buffer: impl Into<index::IndicesSource<'a>>, uniforms: &U, draw_parameters: &DrawParameters<'_>) {
        self.frame.as_mut().unwrap().draw(vertex_buffer.slice(), index_buffer, &shader.program, uniforms, draw_parameters).unwrap();
        self.draw_calls += 1;
    }

    pub fn display(&self) -> &'display Display<WindowSurface> {
        self.display
    }

    pub fn draw_calls(&self) -> u32 {
        self.draw_calls
    }
}

impl<'display> Drop for Frame<'display> {
    fn drop(&mut self) {
        self.frame.take().unwrap().finish().unwrap();
    }
}