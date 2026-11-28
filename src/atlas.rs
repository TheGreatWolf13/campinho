use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::pixel_buffer::PixelBuffer;
use glium::texture::{MipmapsOption, Texture2dArray, UncompressedFloatFormat};
use glium::uniforms::{MagnifySamplerFilter, Sampler};
use glium::{BlitTarget, Display, Rect, Surface};
use log::debug;

pub struct TextureAtlas {
    array: Texture2dArray,
    format: UncompressedFloatFormat,
    capacity: u32,
    width: u32,
    height: u32,
    used: u32,
}

impl TextureAtlas {
    pub fn new(display: &Display<WindowSurface>, format: UncompressedFloatFormat, width: u32, height: u32, capacity: u32) -> Self {
        Self {
            array: Texture2dArray::empty_with_format(display, format, MipmapsOption::EmptyMipmaps, width, height, capacity).unwrap(),
            format,
            width,
            height,
            capacity,
            used: 0,
        }
    }

    pub fn store_texture(&mut self, display: &Display<WindowSurface>, pixels: &[u8], width: u32, height: u32) -> Option<u32> {
        if self.used >= self.capacity {
            debug!("Resized atlas capacity from {} to {}", self.capacity, self.capacity * 2);
            self.recreate(display, self.width, self.height, self.capacity * 2);
        }
        while self.width < width {
            debug!("Resized atlas width from {} to {}", self.width, self.width * 2);
            self.recreate(display, self.width * 2, self.height, self.capacity);
        }
        while self.height < height {
            debug!("Resized atlas height from {} to {}", self.height, self.height * 2);
            self.recreate(display, self.width, self.height * 2, self.capacity);
        }
        let pixel_buffer = PixelBuffer::new_empty(display, pixels.len());
        pixel_buffer.write(&pixels);
        let layer = self.used;
        let mipmap = self.array.mipmap(0).unwrap();
        mipmap.raw_upload_from_pixel_buffer(pixel_buffer.as_slice(), 0..width, 0..height, layer..layer + 1);
        self.used += 1;
        unsafe {
            self.array.generate_mipmaps();
        }
        Some(layer)
    }

    pub fn sampler(&'_ self) -> Sampler<'_, Texture2dArray> {
        self.array.sampled()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn recreate(&mut self, display: &Display<WindowSurface>, new_width: u32, new_height: u32, new_capacity: u32) {
        let new_array = Texture2dArray::empty_with_format(display, self.format, MipmapsOption::EmptyMipmaps, new_width, new_height, new_capacity).unwrap();
        let target = display.draw();
        for layer in 0..self.used {
            let src_fb = SimpleFrameBuffer::new(display, self.array.layer(layer).unwrap().main_level()).unwrap();
            let dst_fb = SimpleFrameBuffer::new(display, new_array.layer(layer).unwrap().main_level()).unwrap();
            let src_rect = Rect {
                left: 0,
                bottom: 0,
                width: self.width,
                height: self.height,
            };
            let dst_rect = BlitTarget {
                left: 0,
                bottom: 0,
                width: self.width as i32,
                height: self.height as i32,
            };
            dst_fb.blit_from_simple_framebuffer(&src_fb, &src_rect, &dst_rect, MagnifySamplerFilter::Nearest);
        }
        target.finish().unwrap();
        self.array = new_array;
        self.width = new_width;
        self.height = new_height;
        self.capacity = new_capacity;
    }
}