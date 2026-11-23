use crate::colour::Colour;
use glium::buffer::WriteMapping;
use glium::glutin::surface::WindowSurface;
use glium::vertex::{VertexBuffer as VB, VertexBufferSlice};
use glium::{implement_vertex, Display, Vertex};
use log::info;

#[derive(Copy, Clone)]
pub struct VertexPosColour {
    pub pos: [f32; 3],
    pub colour: [f32; 4],
}

impl VertexPosColour {
    pub fn new(x: f32, y: f32, color: impl Colour) -> Self {
        Self {
            pos: [x, y, 0.0],
            colour: [color.r(), color.g(), color.b(), color.a()],
        }
    }
}

#[derive(Copy, Clone)]
pub struct VertexPos {
    pub pos: [f32; 3],
}

impl VertexPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: [x, y, 0.0],
        }
    }
}

#[derive(Copy, Clone)]
pub struct VertexBorder {
    pub pos: [f32; 3],
    pub fill_colour: [f32; 4],
    pub border_colour: [f32; 4],
    pub barycenter_basis: [f32; 2],
    pub borders: [f32; 3],
}

implement_vertex!(VertexPos, pos);
implement_vertex!(VertexPosColour, pos, colour);
implement_vertex!(VertexBorder, pos, fill_colour, border_colour, barycenter_basis, borders);

#[derive(Copy, Clone)]
pub enum BitSet3 {
    FFF = 0,
    FFT,
    FTF,
    FTT,
    TFF,
    TFT,
    TTF,
    TTT,
}

impl BitSet3 {
    pub fn test(self, index: u32) -> bool {
        ((self as u8 >> index) & 1) == 1
    }

    pub fn as_array(self) -> [f32; 3] {
        let mut array = [0.0; 3];
        for i in 0..3 {
            if self.test(i) {
                array[i as usize] = f32::INFINITY;
            }
        }
        array
    }
}

pub struct VertexBuffer<T: Vertex> {
    name: &'static str,
    inner: VB<T>,
    used: usize,
    overflow: Vec<T>,
}

pub struct WritableVertexBuffer<'a, 'b, T: Vertex> {
    inner: &'a mut VertexBuffer<T>,
    display: &'b Display<WindowSurface>,
}

pub struct WritableVertexBufferStack<'a, T: Vertex> {
    inner: WriteMapping<'a, [T]>,
    overflow: &'a mut Vec<T>,
    used: &'a mut usize,
}

impl<T: Vertex> VertexBuffer<T> {
    pub fn new(name: &'static str, display: &Display<WindowSurface>, capacity: usize) -> Self {
        Self {
            name,
            inner: VB::empty_dynamic(display, capacity).unwrap(),
            used: 0,
            overflow: Vec::new(),
        }
    }

    pub fn slice(&self) -> VertexBufferSlice<'_, T> {
        self.inner.slice(..self.used).unwrap()
    }

    pub fn write<'a, 'b>(&'a mut self, display: &'b Display<WindowSurface>) -> WritableVertexBuffer<'a, 'b, T> {
        WritableVertexBuffer {
            inner: self,
            display,
        }
    }
}

impl<'a, 'b, T: Vertex> WritableVertexBuffer<'a, 'b, T> {
    pub fn stack(&mut self) -> WritableVertexBufferStack<'_, T> {
        self.inner.used = 0;
        WritableVertexBufferStack {
            inner: self.inner.inner.map_write(),
            used: &mut self.inner.used,
            overflow: &mut self.inner.overflow,
        }
    }
}

impl<'a, 'b, T: Vertex> Drop for WritableVertexBuffer<'a, 'b, T> {
    fn drop(&mut self) {
        if !self.inner.overflow.is_empty() {
            info!("Reallocating {}: buffer from {} to {} vertices", self.inner.name, self.inner.inner.len(), self.inner.used.max(self.inner.inner.len() * 2));
            //Needs reallocation
            let mut new_buffer = VB::empty_dynamic(self.display, self.inner.used.max(self.inner.inner.len() * 2)).unwrap();
            self.inner.inner.copy_to(&new_buffer).unwrap();
            {
                let mut writing = new_buffer.map_write();
                for (i, index) in (self.inner.inner.len()..self.inner.used).enumerate() {
                    writing.set(index, self.inner.overflow[i]);
                }
                self.inner.overflow.clear();
            }
            self.inner.inner = new_buffer;
        }
    }
}

impl<'a, T: Vertex> WritableVertexBufferStack<'a, T> {
    pub fn push(&mut self, v: T) {
        if *self.used < self.inner.len() {
            self.inner.set(*self.used, v);
            *self.used += 1;
        } //
        else {
            self.overflow.push(v);
            *self.used += 1;
        }
    }

    pub fn push_quad(&mut self, v: [T; 4]) {
        self.push(v[0]);
        self.push(v[1]);
        self.push(v[3]);
        self.push(v[3]);
        self.push(v[1]);
        self.push(v[2]);
    }
}

impl <'a> WritableVertexBufferStack<'a, VertexBorder> {
    pub fn push_triangle(&mut self, vertices: [[f32; 3]; 3], fill_colour: impl Colour, border_colour: impl Colour, borders: BitSet3) {
        for i in 0..3 {
            self.push(VertexBorder {
                pos: vertices[i],
                fill_colour: fill_colour.as_array(),
                border_colour: border_colour.as_array(),
                barycenter_basis: todo!(),
                borders: borders.as_array(),
            })
        }
    }
}