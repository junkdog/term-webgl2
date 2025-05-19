use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Glyph {
    /// The glyph ID; used as z-offset in the resulting texture array
    pub id: u16,
    /// The character
    pub symbol: String,
    /// The pixel coordinates of the glyph in the texture
    pub pixel_coords: (i32, i32),
}

#[derive(Debug, Deserialize)]
pub struct FontAtlasConfig {
    /// The font size in points
    pub font_size: f32,
    /// Width of the texture in pixels
    pub texture_width: usize,
    /// Height of the texture in pixels
    pub texture_height: usize,
    /// Width of each character cell
    pub cell_width: i32,
    /// Height of each character cell
    pub cell_height: i32,
    /// The glyphs in the font
    pub glyphs: Vec<Glyph>,
}

impl FontAtlasConfig {
    pub const PADDING: i32 = 1;

    pub fn from_json(json: &str) -> serde_json::Result<FontAtlasConfig> {
        serde_json::from_str(json)
    }

    pub fn terminal_size(
        &self,
        viewport_width: i32,
        viewport_height: i32
    ) -> (i32, i32) {
        (viewport_width / self.cell_width, viewport_height / self.cell_height)
    }

    pub fn cell_size(&self) -> (i32, i32) {
        (self.cell_width, self.cell_height)
    }
}

impl Glyph {
    pub fn new(symbol: char, pixel_coords: (i32, i32)) -> Self {
        // Use a different ID for non-ASCII characters (extended ASCII)
        let id = if symbol as u32 <= 0xff { symbol as u16 } else { 0xFFFF };

        Self {
            id,
            symbol: symbol.to_string(),
            pixel_coords,
        }
    }
}

impl Default for FontAtlasConfig {
    fn default() -> Self {
        Self::from_json(include_str!("../../data/bitmap_font.json"))
            .unwrap()
    }
}
