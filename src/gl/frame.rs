use crate::colour::Colour;
use crate::font::GlyphVertex;
use crate::gl::shader::Shader;
use crate::gl::vertex_buffer::VertexPos;
use crate::gl::vertex_buffer::{VertexBuffer, WritableVertexBuffer};
use crate::seq_literal;
use crate::VertexPosColour;
use glium::glutin::surface::WindowSurface;
use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::{Display, DrawParameters, Surface, Vertex};
use std::mem::ManuallyDrop;

pub struct FrameEngine {
    display: Display<WindowSurface>,
    buffer_holder: BufferHolder,
}

impl FrameEngine {
    pub fn new(display: Display<WindowSurface>) -> Self {
        Self {
            buffer_holder: BufferHolder::new(&display),
            display,
        }
    }

    pub fn start_frame(&'_ mut self) -> Frame<'_> {
        Frame {
            display: &self.display,
            frame: ManuallyDrop::new(self.display.draw()),
            buffer_holder: &mut self.buffer_holder,
            draw_calls: 0,
        }
    }
}

pub struct Frame<'engine> {
    display: &'engine Display<WindowSurface>,
    frame: ManuallyDrop<glium::Frame>,
    buffer_holder: &'engine mut BufferHolder,
    draw_calls: u32,
}

impl<'engine> Frame<'engine> {
    pub fn clear_colour(&mut self, colour: impl Colour) {
        self.frame.clear_color_srgb(colour.r(), colour.g(), colour.b(), colour.a());
    }

    #[allow(private_bounds)]
    pub fn draw<'a, V: Vertex, U: Uniforms>(&mut self, shader: &Shader<V, U>, index_buffer: impl Into<IndicesSource<'a>>, uniforms: &U, draw_parameters: &DrawParameters<'_>)
    where
        BufferHolder: GetBuffer<V>,
    {
        self.frame.draw(self.buffer_holder.get_buffer().slice(), index_buffer, &shader.program, uniforms, draw_parameters).unwrap();
        self.draw_calls += 1;
    }

    pub fn display(&self) -> &'engine Display<WindowSurface> {
        &self.display
    }

    pub fn draw_calls(&self) -> u32 {
        self.draw_calls
    }
}

impl<'engine> Drop for Frame<'engine> {
    fn drop(&mut self) {
        //SAFETY: this ManuallyDrop will not be used again, as the value is dropped
        unsafe {
            ManuallyDrop::take(&mut self.frame).finish().unwrap();
        }
    }
}

trait GetBuffer<V: Vertex> {
    fn get_buffer(&mut self) -> &mut VertexBuffer<V>;
}

seq_literal!(N in ("VertexPos", "VertexPosColour", "GlyphVertex") {
    struct BufferHolder {
        #(
            [<N:snake>]: VertexBuffer<[<N>]>,
        )*
    }

    impl BufferHolder {
        fn new(display: &Display<WindowSurface>) -> Self {
            Self {
                #(
                    [<N:snake>]: VertexBuffer::new(stringify!([<N>]), display, 64),
                )*
            }
        }
    }

    #(
        impl GetBuffer<[<N>]> for BufferHolder {
            fn get_buffer(&mut self) -> &mut VertexBuffer<[<N>]> {
                &mut self.[<N:snake>]
            }
        }
    )*

    impl <'engine> Frame<'engine> {
        #(
            pub fn [<N:snake _buffer>]<'frame>(&'frame mut self) -> WritableVertexBuffer<'frame, 'frame, [<N>]> {
                self.buffer_holder.[<N:snake>].write(&self.display)
            }
        )*
    }
});