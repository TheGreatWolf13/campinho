use glium::buffer::WriteMapping;
use glium::glutin::surface::WindowSurface;
use glium::vertex::{VertexBuffer as VB, VertexBufferSlice};
use glium::{Display, Vertex};
use log::info;

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