use std::collections::HashSet;
use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Style, SwashCache, Weight};
use unicode_segmentation::UnicodeSegmentation;
use font_atlas::{FontAtlasData, FontStyle, Glyph};
use crate::{BitmapFont, PADDING};

const WHITE: Color = Color::rgb(0xff, 0xff, 0xff);

pub(super) struct BitmapFontGenerator {
    font_system: FontSystem,
    cache: SwashCache,
    font_size: f32,
    metrics: Metrics,
    texture_width: i32,
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

        assert!(graphemes.len() <= 512, "Too many unique graphemes: {}", graphemes.len());

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
                // glyphs.push(Glyph::new(c, FontStyle::Normal, (0, 0)));
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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlyphType {
    Normal,
    Emoji
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
            texture_width: next_pow2(slice_width),
            texture_height: next_pow2(slice_height),
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

    fn to_glyph_id(&self) -> u16 {
        self.slice << 4
            | (self.grid_y as u16 * 4) + self.grid_x as u16
    }

    fn xy(&self, config: &RasterizationConfig) -> (i32, i32) {
        let x = self.grid_x as i32 * config.cell_width + PADDING;
        let y = self.grid_y as i32 * config.cell_height + PADDING;
        (x, y)
    }
}

impl BitmapFontGenerator {
    const GRID_SIZE: usize = 4;
    const GLYPHS_PER_SLICE: usize = Self::GRID_SIZE * Self::GRID_SIZE;  // 4x4 grid

    pub fn new(
        font_size: f32,
        texture_width: usize,
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
            texture_width: texture_width as i32,
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
        // Calculate the buffer for this specific glyph
        let mut buffer = self.rasterize_glyph(
            &glyph.symbol,
            glyph.style,
            config.cell_width,
            config.cell_height,
        );

        let mut buffer = buffer.borrow_with(&mut self.font_system);
        let swash_cache = &mut self.cache;

        // todo: special handling for emoji?
        buffer.draw(swash_cache, WHITE, |x, y, w, h, color| {
            if color.a() == 0 || x < 0 || x >= config.cell_width
                || y < 0 || y >= config.cell_height || w != 1 || h != 1 {
                return;
            }

            let x_offset = coord.grid_x as i32 * config.cell_width;
            let y_offset = coord.grid_y as i32 * config.cell_height;

            // calculate position in 3D texture
            let px = x + x_offset + PADDING;
            let py = y + y_offset + PADDING;

            if px >= config.texture_width || py >= config.texture_height {
                return;
            }

            // calculate index in flat array for 3D texture
            let idx = (coord.slice as i32 * config.texture_width * config.texture_height
                + py * config.texture_width + px
            ) as usize;

            if idx < texture.len() {
                let [r, g, b, a] = color.as_rgba().map(|c| c as u32);
                texture[idx] = r << 24 | g << 16 | b << 8 | a;
            }
        });
    }

    fn rasterize_glyph(
        &mut self,
        c: &str,
        style: FontStyle,
        cell_w: i32,
        cell_h: i32,
    ) -> Buffer {
        let f = &mut self.font_system;

        let mut buffer = Buffer::new(f, self.metrics);
        buffer.set_size(f, Some(2.0 * cell_w as f32), Some(2.0 * cell_h as f32));

        buffer.set_text(f, c, &attrs(style), cosmic_text::Shaping::Advanced);
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
        let cell_width = max_width + PADDING * 2;
        let cell_height = max_height + PADDING * 2;

        (cell_width, cell_height)
    }
}

fn sorted_graphemes(chars: &str) -> Vec<&str> {
    let mut graphemes = chars.graphemes(true).collect::<Vec<&str>>();
    graphemes.sort();
    graphemes.dedup();
    graphemes
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