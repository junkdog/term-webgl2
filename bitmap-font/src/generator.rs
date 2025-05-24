use std::collections::HashSet;
use cosmic_text::{Attrs, BorrowedWithFontSystem, Buffer, Color, Family, FontSystem, Metrics, SwashCache, Weight};
use unicode_segmentation::UnicodeSegmentation;
use font_atlas::{FontAtlasConfig, Glyph};
use crate::{BitmapFont, PADDING};

const WHITE: Color = Color::rgb(0xff, 0xff, 0xff);

pub(super) struct BitmapFontGenerator {
    font_system: FontSystem,
    cache: SwashCache,
    font_size: f32,
    metrics: Metrics,
    texture_width: usize,
}

impl BitmapFontGenerator {
    pub fn new(
        font_size: f32,
        texture_width: usize,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let font_db = font_system.db_mut();

        // load the font file
        font_db.load_font_file("./data/NimbusMonoPS-Regular.otf").unwrap();

        let metrics = Metrics::new(font_size, font_size * 1.2);
        let cache = SwashCache::new();

        // initialize with empty glyphs and mp texture height
        Self {
            font_system,
            cache,
            metrics,
            font_size,
            texture_width,
        }
    }

    pub fn generate(
        &mut self,
        chars: &str,
    ) -> BitmapFont {
        let glyphs = chars.graphemes(true).collect::<Vec<&str>>();

        let (cell_w, cell_h) = calculate_cell_dimensions(&mut self.font_system, &mut self.cache, &glyphs, self.metrics);
        let grid_cols = self.texture_width as i32 / cell_w;
        let grid_rows = glyphs.len() as i32 / grid_cols + 1; // assume it's not a perfect fit

        // pad to power-of-2 dimensions
        let texture_height = next_pow2(grid_rows * cell_h);

        let mut texture_data = vec![0; self.texture_width * texture_height as usize];

        let mut glyphs = Vec::new();
        let texture_width = self.texture_width;
        for (i, c) in chars.graphemes(true).enumerate() {
            let i = i as i32;

            if i >= grid_cols * grid_rows { break; }

            let grid_x = i % grid_cols;
            let grid_y = i / grid_cols;

            // calculate pixel positions for this cell
            let pixel_x = grid_x * cell_w;
            let pixel_y = grid_y * cell_h;

            // store id and pixel coordinates for this glyph
            glyphs.push(Glyph::new(c, (pixel_x + PADDING, pixel_y + PADDING)));

            // draw single character to a new buffer
            let mut buffer = self.rasterize_glyph(c, cell_w, cell_h);
            let mut buffer = buffer.borrow_with(&mut self.font_system);

            place_glyph_in_texture(
                &mut self.cache,
                &mut buffer,
                &mut texture_data,
                texture_width,
                pixel_x,
                pixel_y,
                cell_w,
                cell_h,
            );
        }

        assign_missing_glyph_ids(&mut glyphs);

        BitmapFont {
            texture_data,
            metadata: FontAtlasConfig {
                font_size: self.font_size,
                texture_width: texture_width as u32,
                texture_height: texture_height as u32,
                cell_width: cell_w,
                cell_height: cell_h,
                glyphs
            },
        }
    }

    fn rasterize_glyph(
        &mut self,
        c: &str,
        cell_w: i32,
        cell_h: i32,
    ) -> Buffer {
        let mut buffer = Buffer::new(&mut self.font_system, self.metrics);
        // let mut buffer = buffer.borrow_with(&mut self.font_system);
        buffer.set_size(&mut self.font_system, Some(2.0 * cell_w as f32), Some(2.0 * cell_h as f32));

        // add the character to the buffer
        buffer.set_text(&mut self.font_system, c, &attrs(), cosmic_text::Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, true);

        buffer
    }
}

fn assign_missing_glyph_ids(glyphs: &mut [Glyph]) {
    // pre-assigned glyphs (in the range 0x0000-0x00FF)
    let mut used_ids = HashSet::new();
    glyphs.iter()
        .filter(|g| g.id != 0xFFFF)
        .for_each(|g| {
            used_ids.insert(g.id);
        });

    let mut next_id: i32 = -1; // initial value to -1
    let mut next_glyph_id = || {
        let mut id = next_id;
        while id == -1 || used_ids.contains(&(id as u16)) {
            id += 1;
        }

        next_id = id + 1;
        id as u16
    };

    for g in glyphs.iter_mut().filter(|g| g.id == 0xFFFF) {
        g.id = next_glyph_id();
    }

    // sort the glyphs by their ID
    glyphs.sort_by(|a, b| a.id.cmp(&b.id));
}

/// Places a single glyph into the texture at the specified position
fn place_glyph_in_texture(
    swash_cache: &mut SwashCache,
    buffer: &mut BorrowedWithFontSystem<Buffer>,
    texture: &mut [u32],
    texture_width: usize,
    x_offset: i32,
    y_offset: i32,
    width: i32,
    height: i32,
) {
    let texture_width = texture_width as i32;

    buffer.draw(swash_cache, WHITE, |x, y, w, h, color| {
        // alpha is non-zero for the glyph pixels
        let a = color.a();
        if a == 0 || x < 0 || x >= width || y < 0 || y >= height || w != 1 || h != 1 {
            return;
        }

        // calculate the pixel position in the texture
        let px = x + x_offset + PADDING;
        let py = y + y_offset + PADDING;
        if px < 0 || py < 0 || px >= texture_width || py >= texture.len() as i32 / texture_width {
            return;
        }

        let idx = (py * texture_width) + px;
        let [r, g, b, a] = color.as_rgba().map(|c| c as u32);
        texture[idx as usize] = r << 24 | g << 16 | b << 8 | a;
    });
}

/// Calculates the required cell dimensions for a monospaced bitmap font
/// by finding the maximum width and height of all glyphs in the character set.
fn calculate_cell_dimensions(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    chars: &[&str],
    metrics: Metrics
) -> (i32, i32) {
    let mut max_width = 0;
    let mut max_height = 0;

    // create a temporary buffer for measuring
    let width = 100.0;
    let height = 100.0;

    let attrs = attrs();

    // iterate through all characters in the set
    for c in chars {
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        buffer.set_size(Some(width), Some(height));

        // add the character to the buffer, then measure it
        buffer.set_text(&c.to_string(), &attrs, cosmic_text::Shaping::Advanced);

        buffer.draw(swash_cache, WHITE, |x, y, _w, _h, color| {
            let a = color.a();
            if a == 0 || x > width as i32 || y > height as i32 {
                return;
            }

            max_width = x.max(max_width);
            max_height = y.max(max_height);
        });
    }

    // add some padding
    let cell_width = max_width + PADDING * 2;
    let cell_height = max_height + PADDING * 2;

    (cell_width, cell_height)
}

// Rounds up to the next power of 2
fn next_pow2(n: i32) -> i32 {
    let mut v = n;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    v
}

fn attrs() -> Attrs<'static> {
    Attrs::new()
        .family(Family::Monospace)
        .weight(Weight::NORMAL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_pow2() {
        assert_eq!(next_pow2(1), 1);
        assert_eq!(next_pow2(2), 2);
        assert_eq!(next_pow2(3), 4);
        assert_eq!(next_pow2(4), 4);
        assert_eq!(next_pow2(5), 8);
        assert_eq!(next_pow2(15), 16);
        assert_eq!(next_pow2(16), 16);
        assert_eq!(next_pow2(17), 32);
        assert_eq!(next_pow2(1023), 1024);
    }
}