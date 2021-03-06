//! Based on the renderer-texture example in the rust-sdl2 repository.

use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue::Font;
use fontdue_sdl2::FontTexture;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

pub fn main() -> Result<(), String> {
    env_logger::init();
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("fontdue-sdl2 example", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    // fontdue-sdl2:
    let mut font_texture = FontTexture::new(&texture_creator)?;

    // fontdue:
    let font = include_bytes!("roboto/Roboto-Regular.ttf") as &[u8];
    let roboto_regular = Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let fonts = &[roboto_regular];
    let color = Color::RGB(0xFF, 0xFF, 0);
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.append(fonts, &TextStyle::with_user_data("Hello ", 32.0, 0, color));
    layout.append(fonts, &TextStyle::with_user_data("world!", 16.0, 0, color));

    // sdl2 / fontdue-sdl2:
    canvas.clear();
    font_texture.draw_text(&mut canvas, fonts, layout.glyphs())?;
    canvas.present();

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
    }

    Ok(())
}
