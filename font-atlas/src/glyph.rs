use compact_str::{CompactString, ToCompactString};

#[derive(Debug)]
pub struct Glyph {
    /// The glyph ID; used as z-offset in the resulting texture array
    pub id: u16,
    /// The character
    pub symbol: CompactString,
    /// The pixel coordinates of the glyph in the texture
    pub pixel_coords: (i32, i32),
}

impl Glyph {
    pub fn new(symbol: &str, pixel_coords: (i32, i32)) -> Self {
        let first_char = symbol.chars().next().unwrap();
        let id = if symbol.len() == 1 && first_char.is_ascii() {
            // Use a different ID for non-ASCII characters
            first_char as u32 as u16
        } else {
            0xffff
        };

        Self {
            id,
            symbol: symbol.to_compact_string(),
            pixel_coords,
        }
    }
}


