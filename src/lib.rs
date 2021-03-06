use fontdue::layout::{GlyphPosition, GlyphRasterConfig};
use fontdue::Font;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, RenderTarget, Texture, TextureCreator};

mod rect_allocator;
use rect_allocator::{CacheReservation, RectAllocator};

/// A text-rendering-enabled wrapper for [Texture].
pub struct FontTexture<'r> {
    /// The texture containing rendered glyphs in a tightly packed
    /// manner.
    pub texture: Texture<'r>,

    rect_allocator: RectAllocator,
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
        let mut texture = match texture_creator.create_texture_streaming(
            Some(PixelFormatEnum::RGBA8888),
            256,
            256,
        ) {
            Ok(t) => t,
            Err(WidthOverflows(_))
            | Err(HeightOverflows(_))
            | Err(WidthMustBeMultipleOfTwoForFormat(_, _)) => {
                unreachable!()
            }
            Err(SdlError(s)) => return Err(s),
        };
        texture.set_blend_mode(BlendMode::Blend);

        let rect_allocator = RectAllocator::new(256, 256);

        Ok(FontTexture {
            texture,
            rect_allocator,
        })
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
        struct RenderableGlyph {
            key: GlyphRasterConfig,
            texture_rect: Rect,
            canvas_rect: Rect,
        }
        struct MissingGlyph {
            color: Color,
            canvas_rect: Rect,
        }

        let mut result_glyphs = Vec::with_capacity(glyphs.len());
        let mut missing_glyphs = Vec::new();

        for glyph in glyphs
            .into_iter()
            .filter(|glyph| glyph.width * glyph.height > 0)
        {
            let canvas_rect = Rect::new(
                glyph.x as i32,
                glyph.y as i32,
                glyph.width as u32,
                glyph.height as u32,
            );
            let key = glyph.key;
            let color = Color::RGB(0x0, 0x0, 0x0);

            match self.rect_allocator.get_rect_in_texture(*glyph) {
                CacheReservation::AlreadyRasterized(texture_rect) => {
                    result_glyphs.push(RenderableGlyph {
                        key,
                        texture_rect,
                        canvas_rect,
                    });
                }
                CacheReservation::EmptySpace(texture_rect) => {
                    let (_metrics, pixels) = fonts[key.font_index].rasterize_config(key);
                    let color_base = ((color.r as u32) << 24)
                        | ((color.g as u32) << 16)
                        | ((color.b as u32) << 8);

                    if let Err(err) = self.texture.with_lock(texture_rect, |tex_pixels, pitch| {
                        let tex_pixels: &mut [u32] = bytemuck::cast_slice_mut(tex_pixels);
                        let pitch = pitch / 4;
                        for (i, coverage) in pixels.into_iter().enumerate() {
                            let x = i % glyph.width;
                            let y = i / glyph.width;
                            tex_pixels[x + y * pitch] = color_base | coverage as u32;
                        }
                    }) {
                        log::error!(
                            "Error when uploading glyph bitmap of  '{}' (size {}, font index {}): {}",
                            key.c,
                            key.px,
                            key.font_index,
                            err,
                        );
                        missing_glyphs.push(MissingGlyph { color, canvas_rect });
                    } else {
                        result_glyphs.push(RenderableGlyph {
                            key,
                            texture_rect,
                            canvas_rect,
                        });
                    }
                }
                CacheReservation::OutOfSpace => {
                    log::error!(
                        "Glyph cache cannot fit '{}' (size {}, font index {})",
                        key.c,
                        key.px,
                        key.font_index,
                    );
                    missing_glyphs.push(MissingGlyph { color, canvas_rect });
                }
            }
        }

        for glyph in result_glyphs {
            if let Err(err) = canvas.copy(&self.texture, glyph.texture_rect, glyph.canvas_rect) {
                log::error!(
                    "Error when copying glyph '{}' (size {}, font index {}) to canvas: {}",
                    glyph.key.c,
                    glyph.key.px,
                    glyph.key.font_index,
                    err,
                );
            }
        }

        let previous_color = canvas.draw_color();
        for glyph in missing_glyphs {
            canvas.set_draw_color(glyph.color);
            let _ = canvas.draw_rect(glyph.canvas_rect);
        }
        canvas.set_draw_color(previous_color);

        Ok(())
    }
}
