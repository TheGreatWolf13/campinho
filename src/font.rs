use crate::atlas::TextureAtlas;
use crate::colour::Colour;
use crate::font::BakedGlyph::{Empty, Glyph};
use crate::gl::shader::Shader;
use crate::gl::vertex_buffer::{VertexBuffer, WritableVertexBufferStack};
use crate::{impl_uniforms, Frame};
use ab_glyph::{point, Font, FontVec, GlyphId, PxScale, PxScaleFont, Rect, ScaleFont};
use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use glium::texture::{Texture2dArray, UncompressedFloatFormat};
use glium::uniforms::{EmptyUniforms, Sampler, UniformsStorage};
use glium::{implement_vertex, uniform, Blend, Display, DrawParameters};
use log::warn;
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;

const SCALE: f32 = 1024.0;

pub struct FontSystem {
    renderer: FontRenderer,
    atlas: TextureAtlas,
    glyphs: Vec<Option<BakedGlyph>>,
}

struct FontRenderer {
    glyph_shader: Shader<GlyphVertex, GlyphUniform<'static>>,
    font: PxScaleFont<FontVec>,
    glyph_ids_and_advances: HashMap<u32, (GlyphId, f32), BuildNoHashHasher<u32>>,
}

#[derive(Copy, Clone)]
pub enum HorizontalAlignment {
    Left(f32),
    Center(f32),
    Right(f32),
}

#[derive(Copy, Clone)]
pub enum VerticalAlignment {
    Top(f32),
    Center(f32),
    Bottom(f32),
}

enum BakedGlyph {
    Empty,
    Glyph(u32, Rect),
}

impl FontSystem {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let mut vec = vec![];
        vec.resize_with(u16::MAX as usize + 1, || None);
        Self {
            glyphs: vec,
            atlas: TextureAtlas::new(display, UncompressedFloatFormat::U8, 256, 256, 8),
            renderer: FontRenderer::new(display),
        }
    }

    pub fn render_text(&mut self, frame: &mut Frame, buffer: &mut VertexBuffer<GlyphVertex>, text: &str, hor_align: HorizontalAlignment, ver_align: VerticalAlignment, scale: f32, colour: impl Colour, proj: impl Into<[[f32; 4]; 4]>, view: impl Into<[[f32; 4]; 4]>) {
        let display = frame.display();
        let mut batch = self.render_in_batch(frame, colour, proj, view);
        {
            let mut writer = buffer.write(display);
            let mut stack = writer.stack();
            batch.render_text(text, hor_align, ver_align, scale, &mut stack, display);
        }
        batch.end_batch(&buffer);
    }

    pub fn render_in_batch<'font_system, 'frame, 'display, C: Colour, P: Into<[[f32; 4]; 4]>, V: Into<[[f32; 4]; 4]>>(&'font_system mut self, frame: &'frame mut Frame<'display>, colour: C, proj: P, view: V) -> FontRenderBatch<'font_system, 'frame, 'display, C, P, V> {
        FontRenderBatch {
            renderer: &mut self.renderer,
            glyphs: &mut self.glyphs,
            atlas: &mut self.atlas,
            proj,
            view,
            colour,
            frame,
        }
    }

    pub fn width_and_height(&mut self, text: &str, scale: f32, display: &Display<WindowSurface>) -> (f32, f32) {
        self.renderer.width_and_height(text, scale, &mut self.glyphs, &mut self.atlas, display)
    }
}

impl FontRenderer {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let vs = r#"
            #version 140

            in vec2 pos;
            in vec2 uv;
            in uint layer;

            uniform mat4 proj;
            uniform mat4 view;

            out vec2 v_uv;
            flat out uint v_layer;

            void main() {
                v_uv = uv;
                v_layer = layer;
                gl_Position = proj * view * vec4(pos, 0.0, 1.0);
            }
        "#;
        let fs = r#"
            #version 140

            in vec2 v_uv;
            flat in uint v_layer;

            uniform sampler2DArray tex;
            uniform vec4 colour;

            out vec4 frag_colour;

            void main() {
                frag_colour = vec4(colour.r, colour.g, colour.b, colour.a * texture(tex, vec3(v_uv, v_layer)).r);
            }
        "#;
        Self {
            glyph_shader: Shader::new(display, vs, fs),
            font: FontVec::try_from_vec(Vec::from(include_bytes!("../res/JetBrainsMono-Regular.ttf"))).unwrap().into_scaled(PxScale::from(SCALE)),
            glyph_ids_and_advances: HashMap::with_hasher(BuildNoHashHasher::new()),
        }
    }

    fn bake_glyphs(&mut self, text: &str, glyphs: &mut Vec<Option<BakedGlyph>>, atlas: &mut TextureAtlas, display: &Display<WindowSurface>) -> f32 {
        let mut x_span = 0.0;
        for c in text.chars() {
            let (id, advance) = *self.glyph_ids_and_advances.entry(c as u32).or_insert_with_key(|c| {
                let c = unsafe {
                    char::from_u32_unchecked(*c)
                };
                let id = self.font.glyph_id(c);
                (id, self.font.h_advance(id))
            });
            glyphs[id.0 as usize].get_or_insert_with(|| {
                let glyph = self.font.outline_glyph(id.with_scale_and_position(PxScale::from(SCALE), point(0.0, 0.0)));
                if let Some(glyph) = glyph {
                    let bounds = glyph.px_bounds();
                    let w = bounds.width() as usize;
                    let h = bounds.height() as usize;
                    let mut pixels = vec![0; w * h];
                    glyph.draw(|x, y, coverage| {
                        let px = y as usize * w + x as usize;
                        pixels[px] = (coverage * 255.0) as u8;
                    });
                    let layer = atlas.store_texture(display, &pixels, w as u32, h as u32);
                    if let Some(layer) = layer {
                        Glyph(layer, bounds)
                    } //
                    else {
                        warn!("Glyph for '{c}' not stored!");
                        Empty
                    }
                } //
                else {
                    if c != ' ' {
                        warn!("Glyph for '{c}' not found!");
                    }
                    Empty
                }
            });
            x_span += advance;
        }
        x_span
    }

    fn width_and_height(&mut self, text: &str, scale: f32, glyphs: &mut Vec<Option<BakedGlyph>>, atlas: &mut TextureAtlas, display: &Display<WindowSurface>) -> (f32, f32) {
        let x_spam = self.bake_glyphs(text, glyphs, atlas, display) * (scale / SCALE / 2.0);
        let y_spam = scale / 2.0;
        (x_spam, y_spam)
    }

    fn render_text(&mut self, text: &str, hor_align: HorizontalAlignment, ver_align: VerticalAlignment, scale: f32, stack: &mut WritableVertexBufferStack<GlyphVertex>, glyphs: &mut Vec<Option<BakedGlyph>>, atlas: &mut TextureAtlas, display: &Display<WindowSurface>) {
        let x_span = self.bake_glyphs(text, glyphs, atlas, display);
        let scale = scale / SCALE / 2.0;
        let mut x = match hor_align {
            HorizontalAlignment::Left(x) => x,
            HorizontalAlignment::Center(x) => x - x_span / 2.0 * scale,
            HorizontalAlignment::Right(x) => x - x_span * scale,
        };
        let y = match ver_align {
            VerticalAlignment::Top(y) => y - (SCALE * scale),
            VerticalAlignment::Center(y) => y - (SCALE * scale / 2.0),
            VerticalAlignment::Bottom(y) => y,
        };
        for c in text.chars() {
            let (id, advance) = self.glyph_ids_and_advances[&(c as u32)];
            let glyph = &glyphs[id.0 as usize];
            let glyph = glyph.as_ref().unwrap();
            if let Glyph(layer, bounds) = glyph {
                {
                    let x_size = bounds.width() / atlas.width() as f32;
                    let y_size = bounds.height() / atlas.height() as f32;
                    stack.push(GlyphVertex::new(x + bounds.min.x * scale, y - bounds.min.y * scale, 0.0, 0.0, *layer));
                    stack.push(GlyphVertex::new(x + bounds.max.x * scale, y - bounds.min.y * scale, x_size, 0.0, *layer));
                    stack.push(GlyphVertex::new(x + bounds.max.x * scale, y - bounds.max.y * scale, x_size, y_size, *layer));
                    stack.push(GlyphVertex::new(x + bounds.min.x * scale, y - bounds.min.y * scale, 0.0, 0.0, *layer));
                    stack.push(GlyphVertex::new(x + bounds.max.x * scale, y - bounds.max.y * scale, x_size, y_size, *layer));
                    stack.push(GlyphVertex::new(x + bounds.min.x * scale, y - bounds.max.y * scale, 0.0, y_size, *layer));
                }
            }
            x += advance * scale;
        }
    }
}

#[derive(Clone, Copy)]
pub struct GlyphVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    layer: u32,
}

impl GlyphVertex {
    fn new(x: f32, y: f32, u: f32, v: f32, layer: u32) -> Self {
        GlyphVertex {
            pos: [x, y],
            uv: [u, v],
            layer,
        }
    }
}

implement_vertex!(GlyphVertex, pos, uv, layer);

pub struct FontRenderBatch<'font_system, 'frame, 'display, C: Colour, P: Into<[[f32; 4]; 4]>, V: Into<[[f32; 4]; 4]>> {
    renderer: &'font_system mut FontRenderer,
    glyphs: &'font_system mut Vec<Option<BakedGlyph>>,
    atlas: &'font_system mut TextureAtlas,
    proj: P,
    view: V,
    colour: C,
    frame: &'frame mut Frame<'display>,
}

impl<'font_system, 'frame, 'display, C: Colour, P: Into<[[f32; 4]; 4]>, V: Into<[[f32; 4]; 4]>> FontRenderBatch<'font_system, 'frame, 'display, C, P, V> {
    pub fn render_text(&mut self, text: &str, hor_align: HorizontalAlignment, ver_align: VerticalAlignment, scale: f32, stack: &mut WritableVertexBufferStack<GlyphVertex>, display: &Display<WindowSurface>) {
        self.renderer.render_text(text, hor_align, ver_align, scale, stack, &mut self.glyphs, &mut self.atlas, display);
    }

    pub fn end_batch(self, buffer: &VertexBuffer<GlyphVertex>) {
        let mut draw_params = DrawParameters::default();
        draw_params.blend = Blend::alpha_blending();
        let uniforms = GlyphUniform::new(self.proj, self.view, self.colour, self.atlas.sampler());
        self.frame.draw(
            &self.renderer.glyph_shader,
            buffer,
            NoIndices(PrimitiveType::TrianglesList),
            &uniforms,
            &draw_params,
        );
    }
}

pub struct GlyphUniform<'sampler>(UniformsStorage<'static, Sampler<'sampler, Texture2dArray>, UniformsStorage<'static, [f32; 4], UniformsStorage<'static, [[f32; 4]; 4], UniformsStorage<'static, [[f32; 4]; 4], EmptyUniforms>>>>);

impl<'sampler> GlyphUniform<'sampler> {
    pub fn new(proj: impl Into<[[f32; 4]; 4]>, view: impl Into<[[f32; 4]; 4]>, colour: impl Colour, sampler: Sampler<'sampler, Texture2dArray>) -> Self {
        Self(uniform! {
            proj: proj.into(),
            view: view.into(),
            colour: [colour.r(), colour.g(), colour.b(), colour.a()],
            tex: sampler,
        })
    }
}

impl_uniforms!(GlyphUniform<'_>);