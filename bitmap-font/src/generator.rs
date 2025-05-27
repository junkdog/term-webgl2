use std::collections::HashSet;
use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Style, SwashCache, Weight};
use unicode_segmentation::UnicodeSegmentation;
use font_atlas::{FontAtlasConfig, FontStyle, Glyph};
use crate::{BitmapFont, PADDING};

const WHITE: Color = Color::rgb(0xff, 0xff, 0xff);

pub(super) struct BitmapFontGenerator {
    font_system: FontSystem,
    cache: SwashCache,
    font_size: f32,
    metrics: Metrics,
    texture_width: usize,
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

        assert!(graphemes.len() <= 512,"Too many unique graphemes: {}", graphemes.len());

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
    
    fn len(&self) -> usize {
        self.ascii.len() + self.unicode.len() + self.emoji.len()
    }
    
    fn iter(&self) -> impl Iterator<Item = (GlyphType, &str)> {
        let normal = self.ascii.iter().copied()
            .chain(self.unicode.iter().copied())
            .map(|g| (GlyphType::Normal, g));
        let emoji = self.emoji.iter().copied()
            .map(|g| (GlyphType::Emoji, g));
        
        normal.chain(emoji)
    }
}


struct MultiGlyph {
    normal: Glyph,
    bold: Glyph,
    italic: Glyph,
    bold_italic: Glyph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlyphType {
    Normal,
    Emoji
}

impl MultiGlyph {
    
    pub fn id(&self) -> u16 {
        self.normal.id
    }
    
    pub fn set_id(&mut self, id: u16) {
        assert_eq!(id & Glyph::GLYPH_ID_MASK, id);
        
        self.normal.id = id;
        self.bold.id = id | FontStyle::Bold.id_mask(); 
        self.italic.id = id | FontStyle::Italic.id_mask();
        self.bold_italic.id = id | FontStyle::BoldItalic.id_mask();
    }
    
    pub fn flatten(self) -> [Glyph; 4] {
        [
            self.normal,
            self.bold,
            self.italic,
            self.bold_italic,
        ]
    }
}

impl BitmapFontGenerator {
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
            texture_width,
        }
    }

    pub fn generate(&mut self, chars: &str) -> BitmapFont {
        let graphemes = GraphemeSet::new(chars);
        
        let (cell_w, cell_h) = self.calculate_cell_dimensions(&graphemes.unicode);
        
        let grid_cols = self.texture_width as i32 / cell_w;
        let grid_rows = (FontStyle::ALL.len() * graphemes.len()) as i32 / grid_cols + 1; // assume it's not a perfect fit

        // pad to power-of-2 dimensions
        let texture_height = next_pow2(grid_rows * cell_h);

        let mut texture_data = vec![0; self.texture_width * texture_height as usize];

        let styles = FontStyle::ALL.len();
        
        let mut glyphs = Vec::new();
        let mut emojis = Vec::new();
        
        for (i, (glyph_type, c)) in graphemes.iter().enumerate() {
            let mut generate_glyph = |style: FontStyle| {
                let i = (styles * i + style.ordinal()) as i32;

                let grid_x = i % grid_cols;
                let grid_y = i / grid_cols;

                // calculate pixel positions for this cell
                let x = grid_x * cell_w;
                let y = grid_y * cell_h;

                self.place_glyph_in_texture(c, style, &mut texture_data, x, y, cell_w, cell_h,);
                
                Glyph::new(c, style, (x + PADDING, y + PADDING))
            };
        
            match glyph_type {
                GlyphType::Normal => {
                    glyphs.push(MultiGlyph {
                        normal: generate_glyph(FontStyle::Normal),
                        bold: generate_glyph(FontStyle::Bold),
                        italic: generate_glyph(FontStyle::Italic),
                        bold_italic: generate_glyph(FontStyle::BoldItalic),
                    })
                }
                GlyphType::Emoji => {
                    let mut emoji = generate_glyph(FontStyle::Normal);
                    emoji.is_emoji = true;
                    emojis.push(emoji);
                }
            }
        }

        assign_missing_glyph_ids(&mut glyphs);
        assign_emoji_glyph_ids(&mut emojis);

        BitmapFont {
            texture_data,
            metadata: FontAtlasConfig {
                font_size: self.font_size,
                texture_width: self.texture_width as u32,
                texture_height: texture_height as u32,
                cell_width: cell_w,
                cell_height: cell_h,
                glyphs: glyphs.into_iter()
                    .flat_map(|g| g.flatten())
                    .chain(emojis.into_iter())
                    .collect(),
            },
        }
    }

    /// Places a single glyph into the texture at the specified position
    fn place_glyph_in_texture(
        &mut self,
        symbol: &str,
        style: FontStyle,
        texture: &mut [u32],
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
    ) {
        
        let mut buffer = self.rasterize_glyph(symbol, style, width, height);
        let mut buffer = buffer.borrow_with(&mut self.font_system);
        
        let texture_width = self.texture_width as i32;
        let swash_cache = &mut self.cache;
        
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
    fn calculate_cell_dimensions(&mut self, chars: &[&str]) -> (i32, i32) {
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
}

fn sorted_graphemes(chars: &str) -> Vec<&str> {
    let mut graphemes = chars.graphemes(true).collect::<Vec<&str>>();
    graphemes.sort();
    graphemes.dedup();
    graphemes
}

fn assign_emoji_glyph_ids(glyphs: &mut [Glyph]) {
    for (i, glyph) in glyphs.iter_mut().enumerate() {
        glyph.id = i as u16 | Glyph::EMOJI_FLAG;
    }
}

fn assign_missing_glyph_ids(glyphs: &mut [MultiGlyph]) {
    // pre-assigned glyphs (in the range 0x0000-0x00FF)
    let mut used_ids = HashSet::new();
    glyphs.iter()
        .filter(|g| g.id() != Glyph::UNASSIGNED_ID)
        .for_each(|g| {
            used_ids.insert(g.id());
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

    for g in glyphs.iter_mut().filter(|g| g.id() == Glyph::UNASSIGNED_ID) {
        g.set_id(next_glyph_id());
    }

    // sort the glyphs by their ID
    glyphs.sort_by(|a, b| a.id().cmp(&b.id()));
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