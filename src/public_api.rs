use crate::RectAllocator;
use fontdue::layout::GlyphPosition;
use fontdue::Font;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};

/// A text-rendering-enabled wrapper for [`Texture`].
pub struct FontTexture<'r> {
    /// The texture containing rendered glyphs in a tightly packed
    /// manner.
    pub texture: Texture<'r>,
    rect_allocator: RectAllocator,
}

impl FontTexture<'_> {
    /// Creates a new [`FontTexture`] for rendering text.
    ///
    /// Consider the lifetimes of this structure and the given
    /// [`TextureCreator`] as you would a [`Texture`] created with
    /// one, that is why this structure is named "FontTexture".
    ///
    /// # Important note
    ///
    /// Only use a single `&[Font]` for each [`FontTexture`]. Glyphs
    /// with the same index but different font are hard to
    /// differentiate, so using different sets of Fonts when rendering
    /// with a single FontTexture will lead to wrong results.
    ///
    /// # Errors
    ///
    /// The function will return an error if the Texture can't be
    /// created, and the Err(String) will contain an error string from
    /// SDL.
    pub fn new<T>(texture_creator: &TextureCreator<T>) -> Result<FontTexture, String> {
        let texture = crate::create_font_texture(texture_creator)?;
        let rect_allocator = RectAllocator::new(1024, 1024);
        Ok(FontTexture {
            texture,
            rect_allocator,
        })
    }

    /// Renders text to the given canvas, using the given fonts and
    /// glyphs.
    ///
    /// The canvas should be the same one that the [`TextureCreator`]
    /// used in [`FontTexture::new`] was created from.
    ///
    /// The font-slice should be the same one that is passed to
    /// [`Layout::append`](fontdue::layout::Layout::append).
    ///
    /// The glyphs should be from
    /// [`Layout::glyphs`](fontdue::layout::Layout::glyphs).
    ///
    /// # Errors
    ///
    /// This function will return an error if the Texture cannot be
    /// written to, or a copy from the texture to the canvas
    /// fails. This should only really happen under very exceptional
    /// circumstances, so text rendering is interrupted by these
    /// errors. The Err(String) will contain an informational string
    /// from SDL.
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
