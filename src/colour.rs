use crate::gl::frame::Shader;
use crate::gl::vertex_buffer::VertexPos;

pub trait Colour: Copy {
    fn r(&self) -> f32;

    fn g(&self) -> f32;

    fn b(&self) -> f32;

    fn a(&self) -> f32;
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

pub struct ColourWheelManager {
    shader: Shader<VertexPos>
}