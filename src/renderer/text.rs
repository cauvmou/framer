use std::{collections::{HashMap, HashSet}, sync::Arc};

use crate::font;

pub(crate) struct TextState {
    font_to_codepoints: HashMap<Arc<font::Font>, HashSet<u32>>
}

impl TextState {
    pub fn new() -> Self {
        Self {
            font_to_codepoints: HashMap::new(),
        }
    }

    pub fn draw(&mut self, x: u32, y: u32, text: &str, font: Arc<font::Font>) {
        let blob = harfbuzz::Blob::new_read_only(font.data);
        let mut buffer = harfbuzz::Buffer::with(text);
        buffer.guess_segment_properties();
        let glyph_sequence = GlyphSequence {
            glyphs: Self::shape(blob, buffer),
            x,
            y,
            font,
        };
    }

    fn shape(blob: harfbuzz::Blob, buffer: harfbuzz::Buffer) -> Vec<Glyph> {
        unsafe {
            harfbuzz::sys::hb_shape(
                harfbuzz::sys::hb_font_create(harfbuzz::sys::hb_face_create(blob.as_raw(), 0)),
                buffer.as_ptr(),
                std::ptr::null(),
                0,
            );
            let mut glyph_count: u32 = 0;
            let glyph_info = harfbuzz::sys::hb_buffer_get_glyph_infos(
                buffer.as_ptr(),
                (&mut glyph_count) as *mut u32,
            );
            let glyph_pos = harfbuzz::sys::hb_buffer_get_glyph_positions(
                buffer.as_ptr(),
                (&mut glyph_count) as *mut u32,
            );
            let glyph_info = &*std::ptr::slice_from_raw_parts_mut(glyph_info, glyph_count as usize);
            let glyph_pos = &*std::ptr::slice_from_raw_parts_mut(glyph_pos, glyph_count as usize);
            (0..glyph_count)
                .map(|i| Glyph {
                    codepoint: glyph_info[i as usize].codepoint,
                    x_advance: glyph_pos[i as usize].x_advance,
                    y_advance: glyph_pos[i as usize].y_advance,
                    x_offset: glyph_pos[i as usize].x_offset,
                    y_offset: glyph_pos[i as usize].y_offset,
                })
                .collect()
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {

    }
}

pub(crate) struct Glyph {
    codepoint: u32,
    x_advance: i32,
    y_advance: i32,
    x_offset: i32,
    y_offset: i32,
}

pub(crate) struct GlyphSequence {
    glyphs: Vec<Glyph>,
    x: u32,
    y: u32,
    font: Arc<font::Font>,
}
