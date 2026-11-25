use crate::gl::frame::Frame;
use crate::gl::shader::{Shader, Uniform};
use crate::gl::vertex_buffer::{VertexBuffer, VertexPos};
use crate::impl_uniforms;
use cgmath::{ortho, Matrix4, Point3, Vector3};
use glium::glutin::surface::WindowSurface;
use glium::index::NoIndices;
use glium::index::PrimitiveType::TrianglesList;
use glium::uniforms::EmptyUniforms;
use glium::winit::dpi::PhysicalSize;
use glium::{uniform, Display, DrawParameters};

pub trait Colour: Copy {
    fn r(&self) -> f32;

    fn g(&self) -> f32;

    fn b(&self) -> f32;

    fn a(&self) -> f32;

    fn as_array(self) -> [f32; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }
}

#[derive(Copy, Clone)]
pub struct ColourU8(pub u8, pub u8, pub u8, pub u8);

impl Colour for ColourU8 {
    fn r(&self) -> f32 {
        self.0 as f32 / 255.0
    }

    fn g(&self) -> f32 {
        self.1 as f32 / 255.0
    }

    fn b(&self) -> f32 {
        self.2 as f32 / 255.0
    }

    fn a(&self) -> f32 {
        self.3 as f32 / 255.0
    }
}

#[derive(Copy, Clone)]
pub struct ColourHex(pub u32);

impl Colour for ColourHex {
    fn r(&self) -> f32 {
        ((self.0 >> 16) & 0xFF) as f32 / 255.0
    }

    fn g(&self) -> f32 {
        ((self.0 >> 8) & 0xFF) as f32 / 255.0
    }

    fn b(&self) -> f32 {
        ((self.0 >> 0) & 0xFF) as f32 / 255.0
    }

    fn a(&self) -> f32 {
        ((self.0 >> 24) & 0xFF) as f32 / 255.0
    }
}

pub struct ColourWheelManager {
    shader: Shader<VertexPos, ColourWheelUniform>,
}

impl ColourWheelManager {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let vs = r#"
            #version 330

            in vec3 pos;
            out vec2 v_pos;

            uniform mat4 proj;
            uniform mat4 view;

            void main() {
               gl_Position = proj * view * vec4(pos, 1.0);
                v_pos = pos.xy;
            }
        "#;
        let fs = r#"
            #version 330

            in vec2 v_pos;
            out vec4 color;

            uniform vec2 centre;
            uniform float radius;
            uniform float value;

            vec3 hsv2rgb(vec3 c) {
                vec4 k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
                vec3 p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);
                return c.z * mix(k.xxx, clamp(p - k.xxx, 0.0, 1.0), c.y);
            }

            void main() {
                vec2 deltas = v_pos - centre;
                float dist_sqr = deltas.x * deltas.x + deltas.y * deltas.y;
                if (dist_sqr > radius * radius) {
                    discard;
                }
                float hue = atan(deltas.y, -deltas.x) / (2 * 3.141592653589);
                float sat = sqrt(dist_sqr) / radius;
                vec3 rgb = hsv2rgb(vec3(hue, sat, value));
                color = vec4(rgb, 1.0);
            }
        "#;
        let shader = Shader::new(&display, vs, fs, None);
        Self {
            shader
        }
    }

    pub fn draw(&self, frame: &mut Frame, size: PhysicalSize<u32>) {
        let display = frame.display();
        let mut buffer = VertexBuffer::new("Color Wheel", display, 6);
        {
            let mut writer = buffer.write(display);
            let mut stack = writer.stack();
            stack.push_quad([
                VertexPos::new(-25.0, 25.0),
                VertexPos::new(-25.0, -25.0),
                VertexPos::new(25.0, -25.0),
                VertexPos::new(25.0, 25.0),
            ]);
        }
        let aspect_ratio = size.width as f32 / size.height as f32;
        let proj_matrix = ortho(
            -aspect_ratio * 50.0,
            aspect_ratio * 50.0,
            -50.0,
            50.0,
            100.0,
            -100.0,
        );
        let eye = Point3::new(0.0, 0.0, 100.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::unit_y();
        let view_matrix = Matrix4::look_at_rh(eye, target, up);
        let uniforms = ColourWheelUniform::new(proj_matrix, view_matrix, [0.0, 0.0], 25.0, 1.0);
        frame.draw(&self.shader, &buffer, NoIndices(TrianglesList), &uniforms, &DrawParameters::default());
    }
}

struct ColourWheelUniform(Uniform<f32, Uniform<f32, Uniform<[f32; 2], Uniform<[[f32; 4]; 4], Uniform<[[f32; 4]; 4], EmptyUniforms>>>>>);

impl ColourWheelUniform {
    pub fn new(proj: impl Into<[[f32; 4]; 4]>, view: impl Into<[[f32; 4]; 4]>, centre: impl Into<[f32; 2]>, radius: f32, value: f32) -> Self {
        Self(uniform! {
            proj: proj.into(),
            view: view.into(),
            centre: centre.into(),
            radius: radius,
            value: value
        })
    }
}

impl_uniforms!(ColourWheelUniform);