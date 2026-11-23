use crate::impl_uniforms;
use glium::glutin::surface::WindowSurface;
use glium::uniforms::{EmptyUniforms, Uniforms, UniformsStorage};
use glium::{uniform, Display, Program, Vertex};
use std::marker::PhantomData;

pub struct Shader<V: Vertex, U: Uniforms> {
    pub(in crate::gl) program: Program,
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

pub type Uniform<U, R> = UniformsStorage<'static, U, R>;

pub struct UniformProjView(Uniform<[[f32; 4]; 4], Uniform<[[f32; 4]; 4], EmptyUniforms>>);

impl UniformProjView {
    pub fn new(proj: impl Into<[[f32; 4]; 4]>, view: impl Into<[[f32; 4]; 4]>) -> Self {
        Self(uniform! {
            proj: proj.into(),
            view: view.into(),
        })
    }
}

pub struct UniformProjViewThickness(Uniform<f32, Uniform<[[f32; 4]; 4], Uniform<[[f32; 4]; 4], EmptyUniforms>>>);

impl UniformProjViewThickness {
    pub fn new(proj: impl Into<[[f32; 4]; 4]>, view: impl Into<[[f32; 4]; 4]>, thickness: f32) -> Self {
        Self(uniform! {
           proj: proj.into(),
            view: view.into(),
            thickness: thickness,
        })
    }
}

impl_uniforms!(UniformProjView);
impl_uniforms!(UniformProjViewThickness);

#[macro_export]
macro_rules! impl_uniforms {
    ($name:ty) => {
        impl glium::uniforms::Uniforms for $name {
            fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, f: F) {
                self.0.visit_values(f)
            }
        }
    }
}