use crate::RectAllocator;
use fontdue::layout::GlyphPosition;
use fontdue::Font;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};

pub struct FontTexture {
    pub texture: Texture,
    rect_allocator: RectAllocator,
}

impl FontTexture {
    pub fn new<T>(texture_creator: &TextureCreator<T>) -> Result<FontTexture, String> {
        let texture = crate::create_font_texture(texture_creator)?;
        let rect_allocator = RectAllocator::new(1024, 1024);
        Ok(FontTexture {
            texture,
            rect_allocator,
        })
    }

    pub fn draw_text<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        fonts: &[Font],
        glyphs: &[GlyphPosition<Color>],
    ) -> Result<(), String> {
        crate::draw_text(
            &mut self.texture,
            &mut self.rect_allocator,
            canvas,
            fonts,
            glyphs,
        )
    }
}
