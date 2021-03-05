use fontdue::layout::GlyphPosition;
use fontdue::Font;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};

/// A text-rendering-enabled wrapper for [Texture].
pub struct FontTexture<'r> {
    /// The texture containing rendered glyphs in a tightly packed
    /// manner.
    pub texture: Texture<'r>,
}

impl FontTexture<'_> {
    /// Creates a new [FontTexture] for rendering text.
    ///
    /// Consider the lifetimes of this structure and the given
    /// [TextureCreator] as you would a
    /// [Texture] created with one, that is why this
    /// structure is named "FontTexture".
    ///
    /// # Important note
    ///
    /// Only use a single set of [Font]s for each
    /// [FontTexture]. Glyphs with the same index but different font
    /// are hard to differentiate, so using different sets of Fonts
    /// when rendering with a single FontTexture will lead to wrong
    /// results.
    pub fn new<'r, T>(texture_creator: &'r TextureCreator<T>) -> Result<FontTexture<'r>, String> {
        use sdl2::render::TextureValueError::*;
        let texture = match texture_creator.create_texture_streaming(None, 256, 256) {
            Ok(t) => t,
            Err(WidthOverflows(_))
            | Err(HeightOverflows(_))
            | Err(WidthMustBeMultipleOfTwoForFormat(_, _)) => {
                unreachable!()
            }
            Err(SdlError(s)) => return Err(s),
        };

        Ok(FontTexture { texture })
    }

    /// Renders text to the given canvas, using the given fonts and
    /// glyphs.
    ///
    /// The canvas should be the same one that the [TextureCreator]
    /// used in [FontTexture::new] was created from.
    ///
    /// The font-slice should be the same one that is passed to
    /// [Layout::append](fontdue::layout::Layout::append).
    ///
    /// The glyphs should be from
    /// [Layout::glyphs](fontdue::layout::Layout::glyphs).
    pub fn draw_text<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        fonts: &[Font],
        glyphs: &[GlyphPosition],
    ) -> Result<(), String> {
        for glyph in glyphs {
            let (_metrics, pixels) = fonts[glyph.key.font_index].rasterize_config(glyph.key);
            for (i, coverage) in pixels.into_iter().enumerate() {
                let x = (i % glyph.width) as i32 + glyph.x as i32;
                let y = (i / glyph.width) as i32 + glyph.y as i32;
                canvas.set_draw_color(Color::RGBA(0xFF, 0xFF, 0x0, coverage));
                canvas.fill_rect(Rect::new(x, y, 1, 1))?;
            }
        }
        Ok(())
    }
}
