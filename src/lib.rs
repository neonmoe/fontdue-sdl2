//! # fontdue-sdl2
//!
//! This crate is glue code for rendering text with [sdl2][sdl2],
//! rasterized and laid out by [fontdue][fontdue].
//!
//! # Usage
//!
//! First, set up fontdue and layout some glyphs with the [`Color`]
//! included as user data:
//!
//! ```
//! # use fontdue::{Font, layout::{Layout, TextStyle, CoordinateSystem}};
//! # use fontdue_sdl2::FontTexture;
//! # use sdl2::pixels::Color;
//! let font = include_bytes!("../examples/roboto/Roboto-Regular.ttf") as &[u8];
//! let roboto_regular = Font::from_bytes(font, Default::default()).unwrap();
//! let fonts = &[roboto_regular];
//! let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
//! layout.append(fonts, &TextStyle::with_user_data(
//!     "Hello, World!",           // The text to lay out
//!     32.0,                      // The font size
//!     0,                         // The font index (Roboto Regular)
//!     Color::RGB(0xFF, 0xFF, 0)  // The color of the text
//! ));
//! ```
//!
//! Then draw them using a [`FontTexture`]. It needs a
//! [`TextureCreator`], just as any SDL [`Texture`].
//!
//! ```
//! # use fontdue::{Font, layout::{Layout, TextStyle, CoordinateSystem}};
//! # use fontdue_sdl2::FontTexture;
//! # use sdl2::pixels::Color;
//! # let sdl_context = sdl2::init().unwrap();
//! # let video_subsystem = sdl_context.video().unwrap();
//! # let window = video_subsystem.window("fontdue-sdl2 example", 800, 600).position_centered().build().unwrap();
//! # let mut canvas = window.into_canvas().build().unwrap();
//! # let texture_creator = canvas.texture_creator();
//! # let font = include_bytes!("../examples/roboto/Roboto-Regular.ttf") as &[u8];
//! # let roboto_regular = Font::from_bytes(font, Default::default()).unwrap();
//! # let fonts = &[roboto_regular];
//! # let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
//! # layout.append(fonts, &TextStyle::with_user_data(
//! #     "Hello, World!",           // The text to lay out
//! #     32.0,                      // The font size
//! #     0,                         // The font index (Roboto Regular)
//! #     Color::RGB(0xFF, 0xFF, 0)  // The color of the text
//! # ));
//! # canvas.clear();
//! let mut font_texture = FontTexture::new(&texture_creator).unwrap();
//! let _ = font_texture.draw_text(&mut canvas, fonts, layout.glyphs());
//! # canvas.present();
//! ```
//!
//! Note that drawing text can fail if there are issues with the
//! rendering setup. It's often valid to simply ignore, or crash.
//!
//! The [`FontTexture`] is intended to be created once, at the
//! beginning of your program, and then used throughout. Generally,
//! you should treat [`FontTexture`] similarly to the [`Font`]-slice
//! passed to fontdue, and associate each [`FontTexture`] with a
//! specific `&[Font]`. See the [`FontTexture`] documentation for more
//! information.
//!
//! See `examples/basic.rs` for a complete example program.
//!
//! [fontdue]: https://docs.rs/fontdue
//! [sdl2]: https://docs.rs/sdl2

use fontdue::layout::GlyphPosition;
use fontdue::Font;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, RenderTarget, Texture, TextureCreator};

#[cfg(not(feature = "unsafe_textures"))]
mod public_api;

#[cfg(feature = "unsafe_textures")]
mod public_api_no_lifetimes;
#[allow(clippy::unsafe_removed_from_name)]
#[cfg(feature = "unsafe_textures")]
use public_api_no_lifetimes as public_api;

mod rect_allocator;
use rect_allocator::{CacheReservation, RectAllocator};

pub use public_api::FontTexture;

/// Called by [FontTexture::new].
pub(crate) fn create_font_texture<T>(
    texture_creator: &TextureCreator<T>,
) -> Result<Texture, String> {
    use sdl2::render::TextureValueError::*;
    let mut texture = match texture_creator.create_texture_streaming(
        Some(PixelFormatEnum::RGBA32), // = the pixels are always [r, g, b, a] when read as u8's.
        1024,
        1024,
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
    Ok(texture)
}

/// Called by [FontTexture::draw_text].
fn draw_text<RT: RenderTarget>(
    font_texture: &mut Texture,
    rect_allocator: &mut RectAllocator,
    canvas: &mut Canvas<RT>,
    fonts: &[Font],
    glyphs: &[GlyphPosition<Color>],
) -> Result<(), String> {
    struct RenderableGlyph {
        texture_rect: Rect,
        canvas_rect: Rect,
    }
    struct MissingGlyph {
        color: Color,
        canvas_rect: Rect,
    }

    let mut result_glyphs = Vec::with_capacity(glyphs.len());
    let mut missing_glyphs = Vec::new();

    for glyph in glyphs.iter().filter(|glyph| glyph.width * glyph.height > 0) {
        let canvas_rect = Rect::new(
            glyph.x as i32,
            glyph.y as i32,
            glyph.width as u32,
            glyph.height as u32,
        );
        let color = glyph.user_data;

        match rect_allocator.get_rect_in_texture(*glyph) {
            CacheReservation::AlreadyRasterized(texture_rect) => {
                result_glyphs.push(RenderableGlyph {
                    texture_rect,
                    canvas_rect,
                });
            }
            CacheReservation::EmptySpace(texture_rect) => {
                let (metrics, pixels) = fonts[glyph.font_index].rasterize_config(glyph.key);

                let mut full_color_pixels = Vec::with_capacity(pixels.len());
                for coverage in pixels {
                    full_color_pixels.push(color.r);
                    full_color_pixels.push(color.g);
                    full_color_pixels.push(color.b);
                    full_color_pixels.push(coverage);
                }
                font_texture
                    .update(texture_rect, &full_color_pixels, metrics.width * 4)
                    .map_err(|err| format!("{}", err))?;

                result_glyphs.push(RenderableGlyph {
                    texture_rect,
                    canvas_rect,
                });
            }
            CacheReservation::OutOfSpace => {
                log::error!(
                    "Glyph cache cannot fit '{}' (size {}, font index {})",
                    glyph.parent,
                    glyph.key.px,
                    glyph.font_index,
                );
                missing_glyphs.push(MissingGlyph { color, canvas_rect });
            }
        }
    }

    for glyph in result_glyphs {
        canvas.copy(font_texture, glyph.texture_rect, glyph.canvas_rect)?;
    }

    let previous_color = canvas.draw_color();
    for glyph in missing_glyphs {
        canvas.set_draw_color(glyph.color);
        let _ = canvas.draw_rect(glyph.canvas_rect);
    }
    canvas.set_draw_color(previous_color);

    Ok(())
}
