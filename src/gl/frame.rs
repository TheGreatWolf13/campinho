use crate::gl::vertex_buffer::{VertexBuffer, VertexPosColour};
use glium::glutin::surface::WindowSurface;
use glium::uniforms::{EmptyUniforms, UniformValue, Uniforms, UniformsStorage};
use glium::{index, uniform, Display, DrawParameters, Program, Surface, Vertex};
use std::marker::PhantomData;
use crate::colour::Colour;

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

    pub fn draw<'a, V: Vertex, U: Uniforms>(&mut self, vertex_buffer: &VertexBuffer<V>, index_buffer: impl Into<index::IndicesSource<'a>>, shader: &Shader<V, U>, uniforms: &U, draw_parameters: &DrawParameters<'_>) {
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

///<h1>Shader</h1>
pub struct Shader<V: Vertex, U: Uniforms> {
    program: Program,
    _phantom: PhantomData<V>,
    _phantom2: PhantomData<U>,
}

impl<V: Vertex, U: Uniforms> Shader<V, U> {
    pub fn new(display: &Display<WindowSurface>, vs: &'static str, fs: &'static str) -> Self {
        Self {
            program: Program::from_source(display, vs, fs, None).unwrap(),
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

pub type ShaderDefault = Shader<VertexPosColour, DefaultShaderUniform>;

pub struct DefaultShaderUniform(UniformsStorage<'static, [[f32; 4]; 4], UniformsStorage<'static, [[f32; 4]; 4], EmptyUniforms>>);

impl Uniforms for DefaultShaderUniform {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, f: F) {
        self.0.visit_values(f)
    }
}

impl DefaultShaderUniform {
    pub fn new(proj: [[f32; 4]; 4], view: [[f32; 4]; 4]) -> Self {
        Self(uniform! {
            proj: proj,
            view: view,
        })
    }
}