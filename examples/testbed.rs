//! Based on the renderer-texture example in the rust-sdl2 repository.

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use fontdue_sdl2::FontTexture;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

static TEXT_TITLE: &str = "Typesetting\n";
static TEXT_SUBTITLE: &str = "From Wikipedia, the free encyclopedia\n";
static TEXT_START: &str = "\nTypesetting";
static TEXT_TAIL: &str = " is the composition of text by means of arranging physical type[1] or its digital equivalents. Stored letters and other symbols (called sorts in mechanical systems and glyphs in digital systems) are retrieved and ordered according to a language's orthography for visual display. Typesetting requires one or more fonts (which are widely but erroneously confused with and substituted for typefaces). One significant effect of typesetting was that authorship of works could be spotted more easily, making it difficult for copiers who have not gained permission.[2]\n\n";
static TEXT_MORE_1: &str = "During much of the letterpress era, movable type was composed by hand for each page. Cast metal sorts were composed into words, then lines, then paragraphs, then pages of text and tightly bound together to make up a form, with all letter faces exactly the same \"height to paper\", creating an even surface of type. The form was placed in a press, inked, and an impression made on paper.[3]\n\n";
static TEXT_MORE_2: &str = "During typesetting, individual sorts are picked from a type case with the right hand, and set into a composing stick held in the left hand from left to right, and as viewed by the setter upside down. As seen in the photo of the composing stick, a lower case 'q' looks like a 'd', a lower case 'b' looks like a 'p', a lower case 'p' looks like a 'b' and a lower case 'd' looks like a 'q'. This is reputed to be the origin of the expression \"mind your p's and q's\". It might just as easily have been \"mind your b's and d's\".[4]\n\n";

pub fn main() -> Result<(), String> {
    env_logger::init();
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("fontdue-sdl2 example", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut font_texture = FontTexture::new(&texture_creator)?;

    let regular_font = include_bytes!("roboto/Roboto-Regular.ttf") as &[u8];
    let roboto_regular = Font::from_bytes(regular_font, fontdue::FontSettings::default()).unwrap();
    let bold_font = include_bytes!("roboto/Roboto-Bold.ttf") as &[u8];
    let roboto_bold = Font::from_bytes(bold_font, fontdue::FontSettings::default()).unwrap();
    let title_font = include_bytes!("playfair-display/PlayfairDisplay-Regular.ttf") as &[u8];
    let playfair_display = Font::from_bytes(title_font, fontdue::FontSettings::default()).unwrap();
    let fonts = &[roboto_regular, roboto_bold, playfair_display];
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    let mut layout_settings = LayoutSettings {
        x: 10.0,
        y: 10.0,
        max_width: Some(780.0),
        ..LayoutSettings::default()
    };

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let (width, height) = canvas.output_size().unwrap();
        layout_settings.max_width = Some(width as f32 - 20.0);
        layout.reset(&layout_settings);
        layout.append(fonts, &TextStyle::new(TEXT_TITLE, 30.0, 2));
        layout.append(fonts, &TextStyle::new(TEXT_SUBTITLE, 16.0, 0));
        layout.append(fonts, &TextStyle::new(TEXT_START, 16.0, 1));
        layout.append(fonts, &TextStyle::new(TEXT_TAIL, 16.0, 0));
        layout.append(fonts, &TextStyle::new(TEXT_MORE_1, 64.0, 0));
        layout.append(fonts, &TextStyle::new(TEXT_MORE_2, 12.0, 0));

        canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFE));
        canvas.clear();

        font_texture.draw_text(&mut canvas, fonts, layout.glyphs())?;
        let glyph_cache_rect = Rect::new(width as i32 - 270, height as i32 - 270, 256, 256);
        canvas.set_draw_color(Color::RGB(0xEE, 0xEE, 0xEE));
        let _ = canvas.fill_rect(glyph_cache_rect);
        let _ = canvas.copy(&font_texture.texture, None, glyph_cache_rect);

        canvas.present();
    }

    Ok(())
}
