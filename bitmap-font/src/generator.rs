use std::collections::HashSet;
use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Style, SwashCache, Weight};
use unicode_segmentation::UnicodeSegmentation;
use font_atlas::{FontAtlasData, FontStyle, Glyph};
use crate::BitmapFont;

const WHITE: Color = Color::rgb(0xff, 0xff, 0xff);

pub(super) struct BitmapFontGenerator {
    font_system: FontSystem,
    cache: SwashCache,
    font_size: f32,
    metrics: Metrics,
}


struct GraphemeSet<'a> {
    ascii: Vec<&'a str>,
    unicode: Vec<&'a str>,
    emoji: Vec<&'a str>,
}

impl<'a> GraphemeSet<'a> {
    fn new(chars: &'a str) -> Self {
        let mut graphemes = chars.graphemes(true).collect::<Vec<&str>>();
        graphemes.sort();
        graphemes.dedup();


        let mut ascii = vec![];
        let mut unicode = vec![];
        let mut emoji = vec![];

        for g in graphemes {
            if g.len() == 1 && g.chars().all(|c| c.is_ascii()) {
                ascii.push(g);
            } else if emojis::get(g).is_some() {
                emoji.push(g);
            } else {
                unicode.push(g);
            }
        }
        let non_emoji_glyphs = ascii.len() + unicode.len();
        assert!(non_emoji_glyphs <= 512, "Too many unique graphemes: {}", non_emoji_glyphs);

        Self { ascii, unicode, emoji }
    }

    pub(super) fn into_glyphs(self) -> Vec<Glyph> {
        let mut glyphs = Vec::new();

        // pre-assigned glyphs (in the range 0x000-0x07F)
        let mut used_ids = HashSet::new();
        for c in self.ascii.iter() {
            used_ids.insert(c.chars().next().unwrap() as u16);
            for style in FontStyle::ALL {
                glyphs.push(Glyph::new(c, style, (0, 0)));
            }
        }

        // unicode glyphs fill any gaps in the ASCII range (0x000-0x1FF)
        glyphs.extend(assign_missing_glyph_ids(used_ids, &self.unicode));

        // emoji glyphs are assigned IDs starting from 0x800
        for (i, c) in self.emoji.iter().enumerate() {
            let id = i as u16 | Glyph::EMOJI_FLAG;
            let mut glyph = Glyph::new_with_id(id, c, FontStyle::Normal, (0, 0));
            glyph.is_emoji = true;
            glyphs.push(glyph);
        }

        glyphs.sort_by_key(|g| g.id);

        glyphs
    }
}

#[derive(Debug)]
struct RasterizationConfig {
    texture_width: i32,
    texture_height: i32,
    texture_depth: i32, // slices
    cell_width: i32,
    cell_height: i32,
}

impl RasterizationConfig {
    const GLYPHS_PER_SLICE: i32 = 16; // 4x4 grid
    const GRID_SIZE: i32 = 4;

    fn new(
        cell_width: i32,
        cell_height: i32,
        glyphs: &[Glyph],
    ) -> Self {
        let slice_width = Self::GRID_SIZE * cell_width;
        let slice_height = Self::GRID_SIZE * cell_height;

        let max_id = glyphs.iter().map(|g| g.id).max().unwrap_or(0) as i32;
        let depth = (max_id + Self::GLYPHS_PER_SLICE - 1) / Self::GLYPHS_PER_SLICE;

        Self {
            texture_width: slice_width,
            texture_height: slice_height,
            texture_depth: next_pow2(depth),
            cell_width,
            cell_height,
        }
    }

    fn texture_size(&self) -> usize {
        (self.texture_width * self.texture_height * self.texture_depth) as usize
    }
}

#[derive(Debug, Clone, Copy)]
struct GlyphCoordinate {
    slice: u16, // Z coordinate in 3D texture
    grid_x: u8, // X position in 4x4 grid (0-3)
    grid_y: u8, // Y position in 4x4 grid (0-3)
}

impl GlyphCoordinate {
    fn from_glyph_id(id: u16) -> Self {
        // 16 glyphs per slice (4x4)
        let slice = id >> 4;
        let position_in_slice = id & 0xF;
        let grid_x = (position_in_slice % 4) as u8;
        let grid_y = (position_in_slice / 4) as u8;

        Self { slice, grid_x, grid_y }
    }

    fn xy(&self, config: &RasterizationConfig) -> (i32, i32) {
        let x = self.grid_x as i32 * config.cell_width + FontAtlasData::PADDING;
        let y = self.grid_y as i32 * config.cell_height + FontAtlasData::PADDING;
        (x, y)
    }

    fn cell_offset(&self, config: &RasterizationConfig) -> (i32, i32, i32) {
        (
            self.grid_x as i32 * config.cell_width,
            self.grid_y as i32 * config.cell_height,
            self.slice as i32,
        )
    }
}

impl BitmapFontGenerator {

    pub fn new(
        font_size: f32,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let font_db = font_system.db_mut();

        // load the font files
        font_db.load_font_file("./data/NimbusMonoPS-Regular.otf").unwrap();
        font_db.load_font_file("./data/NimbusMonoPS-Bold.otf").unwrap();
        font_db.load_font_file("./data/NimbusMonoPS-Italic.otf").unwrap();
        font_db.load_font_file("./data/NimbusMonoPS-BoldItalic.otf").unwrap();

        let metrics = Metrics::new(font_size, font_size * 1.2);
        let cache = SwashCache::new();

        Self {
            font_system,
            cache,
            metrics,
            font_size,
        }
    }

    pub fn generate(&mut self, chars: &str) -> BitmapFont {
        // categorize and allocate IDs
        let grapheme_set = GraphemeSet::new(chars);
        let glyphs = grapheme_set.into_glyphs();

        // calculate texture dimensions
        // let (cell_w, cell_h) = self.calculate_cell_dimensions(&glyphs);
        let (cell_w, cell_h) = self.calculate_cell_dimensions(&[Glyph::new("█", FontStyle::Normal, (0, 0))]);
        let config = RasterizationConfig::new(cell_w, cell_h, &glyphs);
        println!("{:?}", &config);

        // allocate 3d rgba texture data
        let mut texture_data = vec![0u32; config.texture_size()];

        // rasterize glyphs into 3d texture
        let mut rasterized_glyphs = Vec::with_capacity(glyphs.len());
        for glyph in glyphs.into_iter() {
            let coord = GlyphCoordinate::from_glyph_id(glyph.id);

            self.place_glyph_in_3d_texture(
                &glyph,
                &config,
                &mut texture_data,
                coord,
            );

            // update glyph with actual texture coordinates
            let mut updated_glyph = glyph;
            updated_glyph.pixel_coords = coord.xy(&config);
            rasterized_glyphs.push(updated_glyph);
        }

        BitmapFont {
            atlas_data: FontAtlasData {
                font_size: self.font_size,
                texture_width: config.texture_width as u32,
                texture_height: config.texture_height as u32,
                texture_depth: config.texture_depth as u32,
                cell_width: config.cell_width,
                cell_height: config.cell_height,
                glyphs: rasterized_glyphs,
                texture_data,
            },
        }
    }

    /// Places a single glyph into the texture at the specified position
    fn place_glyph_in_3d_texture(
        &mut self,
        glyph: &Glyph,
        config: &RasterizationConfig,
        texture: &mut [u32],
        coord: GlyphCoordinate,
    ) {
        let inner_cell_w = config.cell_width - FontAtlasData::PADDING * 2;
        let inner_cell_h = config.cell_height - FontAtlasData::PADDING * 2;

        // rasterize the glyph
        let mut buffer = self.rasterize_glyph_for_atlas(glyph, inner_cell_w, inner_cell_h);
        let buffer_size = self.get_buffer_size(glyph.is_emoji, inner_cell_w, inner_cell_h);

        buffer.set_size(&mut self.font_system, Some(buffer_size.0), Some(buffer_size.1));

        let mut buffer = buffer.borrow_with(&mut self.font_system);
        let cell_offset = coord.cell_offset(config);

        // collect pixels and optionally calculate centering
        let pixels = Self::collect_glyph_pixels(&mut buffer, &mut self.cache, glyph.is_emoji, inner_cell_w, inner_cell_h);

        // render pixels to texture
        let cell_offset = (cell_offset.0, cell_offset.1);
        self.render_pixels_to_texture(pixels, cell_offset, coord.slice as i32, config, texture);
    }

    fn rasterize_glyph_for_atlas(&mut self, glyph: &Glyph, inner_w: i32, inner_h: i32) -> Buffer {
        if glyph.is_emoji {
            self.rasterize_emoji(&glyph.symbol, inner_w as f32, inner_h as f32)
        } else {
            self.rasterize_glyph(&glyph.symbol, glyph.style, inner_w, inner_h)
        }
    }

    fn get_buffer_size(&self, is_emoji: bool, inner_w: i32, inner_h: i32) -> (f32, f32) {
        if is_emoji {
            (inner_w as f32 * 2.0, inner_h as f32 * 2.0)
        } else {
            (inner_w as f32, inner_h as f32)
        }
    }

    fn collect_glyph_pixels(
        buffer: &mut cosmic_text::BorrowedWithFontSystem<Buffer>,
        cache: &mut SwashCache,
        center_emoji: bool,
        inner_w: i32,
        inner_h: i32,
    ) -> Vec<(i32, i32, Color)> {
        let mut pixels = Vec::new();
        let mut bounds = GlyphBounds::new();

        buffer.draw(cache, WHITE, |x, y, w, h, color| {
            if color.a() > 0 && w == 1 && h == 1 {
                bounds.update(x, y);
                pixels.push((x, y, color));
            }
        });

        if center_emoji && !pixels.is_empty() {
            let (offset_x, offset_y) = bounds.centering_offset(inner_w, inner_h);
            pixels.iter_mut().for_each(|(x, y, _)| {
                *x += offset_x;
                *y += offset_y;
            });
        }

        pixels
    }

    fn render_pixels_to_texture(
        &self,
        pixels: Vec<(i32, i32, Color)>,
        cell_offset: (i32, i32),
        slice: i32,
        config: &RasterizationConfig,
        texture: &mut [u32],
    ) {
        let inner_w = config.cell_width - FontAtlasData::PADDING * 2;
        let inner_h = config.cell_height - FontAtlasData::PADDING * 2;

        for (x, y, color) in pixels {
            if x < 0 || x >= inner_w || y < 0 || y >= inner_h {
                continue;
            }

            let px = x + cell_offset.0 + FontAtlasData::PADDING;
            let py = y + cell_offset.1 + FontAtlasData::PADDING;

            if px >= 0 && px < config.texture_width && py >= 0 && py < config.texture_height {
                let idx = self.texture_index(px, py, slice, config);

                if idx < texture.len() {
                    let [r, g, b, a] = color.as_rgba().map(|c| c as u32);
                    texture[idx] = r << 24 | g << 16 | b << 8 | a;
                }
            }
        }
    }

    fn texture_index(&self, x: i32, y: i32, slice: i32, config: &RasterizationConfig) -> usize {
        (slice * config.texture_width * config.texture_height + y * config.texture_width + x) as usize
    }



    fn rasterize_glyph(
        &mut self,
        c: &str,
        style: FontStyle,
        inner_cell_w: i32,
        inner_cell_h: i32,
    ) -> Buffer {
        let f = &mut self.font_system;

        let mut buffer = Buffer::new(f, self.metrics);
        buffer.set_size(f, Some(inner_cell_w as f32), Some(inner_cell_h as f32));

        buffer.set_monospace_width(f, Some(inner_cell_w as f32));
        buffer.set_text(f, c, &attrs(style), cosmic_text::Shaping::Advanced);
        buffer.shape_until_scroll(f, true);

        buffer
    }
    
    fn rasterize_emoji(
        &mut self,
        emoji: &str,
        inner_cell_w: f32,
        inner_cell_h: f32,
    ) -> Buffer {
        let f = &mut self.font_system;

        // First pass: measure at default size
        let measure_size = self.font_size * 2.0; // Start slightly larger
        let measure_metrics = Metrics::new(measure_size, measure_size * 2.0);

        let mut measure_buffer = Buffer::new(f, measure_metrics);
        measure_buffer.set_size(f, Some(inner_cell_w * 4.0), Some(inner_cell_h * 4.0));
        measure_buffer.set_text(f, emoji, &attrs(FontStyle::Normal), cosmic_text::Shaping::Advanced);
        measure_buffer.shape_until_scroll(f, true);

        // Measure actual bounds
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        let mut has_content = false;

        let mut measure_buffer = measure_buffer.borrow_with(f);
        measure_buffer.draw(&mut self.cache, WHITE, |x, y, _w, _h, color| {
            if color.a() > 0 {
                has_content = true;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        });
        drop(measure_buffer);

        if !has_content {
            // Fallback for emojis that don't render
            return self.rasterize_glyph(emoji, FontStyle::Normal, inner_cell_w as i32, inner_cell_h as i32);
        }

        // calculate actual dimensions
        let actual_width = (max_x - min_x + 1) as f32;
        let actual_height = (max_y - min_y + 1) as f32;

        // calculate scale factor (with 80% target to leave some padding)
        let scale_x = (inner_cell_w) / actual_width;
        let scale_y = (inner_cell_h) / actual_height;
        let scale = scale_x.min(scale_y).min(1.0); // Don't scale up

        // render at scaled size
        let scaled_size = measure_size * scale;
        let scaled_metrics = Metrics::new(scaled_size, scaled_size * 1.2);

        let mut buffer = Buffer::new(f, scaled_metrics);
        buffer.set_size(f, Some(inner_cell_w), Some(inner_cell_w));
        buffer.set_text(f, emoji, &attrs(FontStyle::Normal), cosmic_text::Shaping::Advanced);
        buffer.shape_until_scroll(f, true);

        buffer
    }

    /// Calculates the required cell dimensions for a monospaced bitmap font
    /// by finding the maximum width and height of all glyphs in the character set.
    fn calculate_cell_dimensions(&mut self, glyps: &[Glyph]) -> (i32, i32) {
        let mut max_width = 0;
        let mut max_height = 0;

        // create a temporary buffer for measuring
        let width = 100.0;
        let height = 100.0;

        let attrs = attrs(FontStyle::Normal);

        let font_system = &mut self.font_system;
        let swash_cache = &mut self.cache;
        let metrics = self.metrics;

        // iterate through all characters in the set
        for c in glyps.iter().map(|g| &g.symbol) {
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
        let cell_width = max_width + FontAtlasData::PADDING * 2;
        let cell_height = max_height + FontAtlasData::PADDING * 2;

        (cell_width, cell_height)
    }
}


fn assign_missing_glyph_ids(
    used_ids: HashSet<u16>,
    symbols: &[&str]
) -> Vec<Glyph> {
    let mut next_id: i32 = -1; // initial value to -1
    let mut next_glyph_id = || {
        let mut id = next_id;
        while id == -1 || used_ids.contains(&(id as u16)) {
            id += 1;
        }

        next_id = id + 1;
        id as u16
    };

    symbols.iter()
        .flat_map(|c| {
            let base_id = next_glyph_id();
            [
                Glyph::new_with_id(base_id, c, FontStyle::Normal, (0, 0)),
                Glyph::new_with_id(base_id, c, FontStyle::Bold, (0, 0)),
                Glyph::new_with_id(base_id, c, FontStyle::Italic, (0, 0)),
                Glyph::new_with_id(base_id, c, FontStyle::BoldItalic, (0, 0)),
            ]
        })
        .collect()
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

fn attrs(style: FontStyle) -> Attrs<'static> {
    let attrs = Attrs::new()
        .style(Style::Normal)
        .family(Family::Monospace)
        .weight(Weight::NORMAL);

    use FontStyle::*;
    match style {
        Normal     => attrs,
        Bold       => attrs.weight(Weight::BOLD),
        Italic     => attrs.style(Style::Italic),
        BoldItalic => attrs.style(Style::Italic).weight(Weight::BOLD),
    }
}

struct GlyphBounds {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl GlyphBounds {
    fn new() -> Self {
        Self {
            min_x: i32::MAX,
            max_x: i32::MIN,
            min_y: i32::MAX,
            max_y: i32::MIN,
        }
    }

    fn update(&mut self, x: i32, y: i32) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    fn centering_offset(&self, cell_w: i32, cell_h: i32) -> (i32, i32) {
        let content_w = self.max_x - self.min_x + 1;
        let content_h = self.max_y - self.min_y + 1;

        let offset_x = (cell_w - content_w) / 2 - self.min_x;
        let offset_y = (cell_h - content_h) / 2 - self.min_y;

        (offset_x, offset_y)
    }
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