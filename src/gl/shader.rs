use crate::gl::vertex_buffer::VertexPosColour;
use crate::impl_uniforms;
use glium::glutin::surface::WindowSurface;
use glium::uniforms::{EmptyUniforms, Uniforms, UniformsStorage};
use glium::{uniform, Display, Program, Vertex};
use std::marker::PhantomData;

pub struct ShaderManager {
    line_shader: Shader<VertexPosColour, UniformProjViewThickness>,
    default_shader: Shader<VertexPosColour, UniformProjView>,
}

impl ShaderManager {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        Self {
            line_shader: Shader::new(
                display,
                r#"
                    #version 330

                    in vec3 pos;
                    in vec4 colour;

                    out vec4 v2g_colour;

                    void main() {
                        gl_Position = vec4(pos, 1.0);
                        v2g_colour = colour;
                    }
                "#,
                Some(r#"
                    #version 330

                    layout(lines) in;
                    layout(triangle_strip, max_vertices = 4) out;

                    in vec4 v2g_colour[];

                    out vec4 g2f_colour;

                    uniform mat4 proj;
                    uniform mat4 view;
                    uniform float thickness;

                    void main() {
                        vec4 pos0 = gl_in[0].gl_Position;
                        vec4 pos1 = gl_in[1].gl_Position;
                        vec4 dir = pos1 - pos0;
                        vec4 normal = normalize(vec4(-dir.y, dir.x, 0.0, 0.0));
                        gl_Position = proj * view * (pos0 + normal * 0.025 * thickness);
                        g2f_colour = v2g_colour[0];
                        EmitVertex();
                        gl_Position = proj * view * (pos0 - normal * 0.025 * thickness);
                        g2f_colour = v2g_colour[0];
                        EmitVertex();
                        gl_Position = proj * view * (pos1 + normal * 0.025 * thickness);
                        g2f_colour = v2g_colour[1];
                        EmitVertex();
                        gl_Position = proj * view * (pos1 - normal * 0.025 * thickness);
                        g2f_colour = v2g_colour[1];
                        EmitVertex();
                        EndPrimitive();
                    }
                "#),
                r#"
                    #version 330

                    in vec4 g2f_colour;
                    out vec4 colour;

                    void main() {
                        colour = g2f_colour;
                    }
                "#,
            ),
            default_shader: Shader::new(
                display,
                r#"
                    #version 330

                    in vec3 pos;
                    in vec4 colour;

                    out vec4 v2f_colour;

                    void main() {
                        gl_Position = vec4(pos, 1.0);
                        v2f_colour = colour;
                    }
                "#,
                None,
                r#"
                    #version 330

                    in vec4 v2f_colour;

                    out vec4 colour;

                    void main() {
                        colour = v2f_colour;
                    }
                "#
            ),
        }
    }

    pub fn default(&self) -> &Shader<VertexPosColour, UniformProjView> {
        &self.default_shader
    }

    pub fn line(&self) -> &Shader<VertexPosColour, UniformProjViewThickness> {
        &self.line_shader
    }
}

pub struct Shader<V: Vertex, U: Uniforms> {
    pub(in crate::gl) program: Program,
    _phantom: PhantomData<V>,
    _phantom2: PhantomData<U>,
}

impl<V: Vertex, U: Uniforms> Shader<V, U> {
    pub fn new(display: &Display<WindowSurface>, vs: &'static str, gs: Option<&'static str>, fs: &'static str) -> Self {
        Self {
            program: Program::from_source(display, vs, fs, gs).unwrap(),
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