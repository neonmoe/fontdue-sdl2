# fontdue-sdl2
A crate for drawing text with [sdl2::render][sdl2::render], using
[fontdue][fontdue] for rasterization. This library is mostly
glue-code, all the good parts are from the aforementioned libraries.

The motivation for this crate is to allow easier, rustier, and better
quality text rendering in projects that use SDL2's render module,
compared to sdl2_ttf.

## Documentation

Read the docs on [docs.rs][docs].

## Design decisions

(The library isn't ready yet, but this is the idea I'm working towards.)

- This library consumes fontdue data (such as layout and fonts) and
  then calls sdl2::render to render that on screen. This library only
  owns caches, such as the LayoutCache and a glyph cache.
- Major difference to sdl2_ttf: this library draws each glyph as its
  own quad, from a single gylph cache texture. This is very fast on
  modern GPUs, as it can be done in a single draw call. Per-text-area
  caching can still be achieved by rendering this library's results
  into a render texture.

## Screenshots

These will mostly show off fontdue (the text) and SDL2 (the window),
but I think rendering crates should have screenshots for first
impressions. TODO.

## License

This library can be used under the terms of the [MIT license][license].

[sdl2::render]: https://docs.rs/sdl2/0.34.3/sdl2/render/index.html
[fontdue]: https://crates.io/crates/fontdue
[docs]: https://docs.rs/fontdue-sdl2/
[license]: LICENSE.md
