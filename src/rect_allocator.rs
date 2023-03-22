use fontdue::layout::{GlyphPosition, GlyphRasterConfig};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
struct GlyphKey {
    glyph: GlyphRasterConfig,
    color: Color,
}

pub enum CacheReservation {
    AlreadyRasterized(Rect),
    EmptySpace(Rect),
    OutOfSpace,
}

pub struct RectAllocator {
    empty_rects: Vec<Rect>,
    reserved_rects: HashMap<GlyphKey, Rect>,
}

impl RectAllocator {
    pub fn new(width: u32, height: u32) -> RectAllocator {
        RectAllocator {
            empty_rects: vec![Rect::new(0, 0, width, height)],
            reserved_rects: HashMap::new(),
        }
    }

    pub fn get_rect_in_texture(&mut self, glyph: GlyphPosition<Color>) -> CacheReservation {
        let key = GlyphKey {
            glyph: glyph.key,
            color: glyph.user_data,
        };
        if let Some(already_reserved) = self.reserved_rects.get(&key) {
            CacheReservation::AlreadyRasterized(*already_reserved)
        } else if let Some(new_rect) = self.get_empty_slot(glyph.width as u32, glyph.height as u32)
        {
            self.reserved_rects.insert(key, new_rect);
            CacheReservation::EmptySpace(new_rect)
        } else {
            CacheReservation::OutOfSpace
        }
    }

    fn get_empty_slot(&mut self, width: u32, height: u32) -> Option<Rect> {
        let new_rect = if let Some(rect) = self
            .empty_rects
            .iter_mut().find(|rect| rect.width() >= width && rect.height() >= height)
        {
            let mut new_rect = *rect;
            new_rect.resize(width, height);
            new_rect
        } else {
            return None;
        };

        // Remove entirely contained empty rects:
        self.empty_rects
            .retain(|rect| !new_rect.contains_rect(*rect));

        // Split intersecting rects into surrounding rects:
        // TODO(cleanup): Could use Vec::drain_filter here, once it's stable
        let mut i = 0;
        while i < self.empty_rects.len() {
            if self.empty_rects[i].has_intersection(new_rect) {
                let intersecting_rect = self.empty_rects.remove(i);

                if intersecting_rect.left() < new_rect.left() {
                    let mut new_empty = intersecting_rect;
                    new_empty.set_width((new_rect.left() - intersecting_rect.left()) as u32);
                    debug_assert!(!new_empty.has_intersection(new_rect));
                    self.empty_rects.push(new_empty);
                }

                if intersecting_rect.right() > new_rect.right() {
                    let mut new_empty = intersecting_rect;
                    new_empty.set_width((intersecting_rect.right() - new_rect.right()) as u32);
                    new_empty.set_x(new_rect.right());
                    debug_assert!(!new_empty.has_intersection(new_rect));
                    self.empty_rects.push(new_empty);
                }

                if intersecting_rect.top() < new_rect.top() {
                    let mut new_empty = intersecting_rect;
                    new_empty.set_height((new_rect.top() - intersecting_rect.top()) as u32);
                    debug_assert!(!new_empty.has_intersection(new_rect));
                    self.empty_rects.push(new_empty);
                }

                if intersecting_rect.bottom() > new_rect.bottom() {
                    let mut new_empty = intersecting_rect;
                    new_empty.set_height((intersecting_rect.bottom() - new_rect.bottom()) as u32);
                    new_empty.set_y(new_rect.bottom());
                    debug_assert!(!new_empty.has_intersection(new_rect));
                    self.empty_rects.push(new_empty);
                }
            } else {
                i += 1;
            }
        }

        // TODO(opt): Is the sort & consolidate really needed?
        // TODO: Reclaiming unused areas
        // TODO: Resizing the texture

        // Sort the empty rects by size (smallest first, so small
        // glyphs will fit into the small nooks and crannies if
        // possible)
        self.empty_rects.sort_by_key(|a| a.width() * a.height());

        // Remove rects that are completely within another. Reasoning:
        // this should avoid "fake small areas" that are created
        // inside bigger areas by the splitting algorithm above.
        let mut i = 1;
        while i < self.empty_rects.len() {
            let rect = self.empty_rects[i];
            let mut j = 0;
            while j < i {
                if rect.contains_rect(self.empty_rects[j]) {
                    self.empty_rects.remove(j);
                    i -= 1;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }

        Some(new_rect)
    }
}
