use compact_str::{CompactString, ToCompactString};

/// Represents a single character glyph in a font atlas texture.
///
/// A `Glyph` contains the metadata needed to locate and identify a character
/// within a font atlas texture. Each glyph has a unique ID that corresponds
/// to its layer in a WebGL 2D texture array, along with its pixel coordinates
/// in the source atlas image.
///
/// # ASCII Optimization
/// For ASCII characters, the glyph ID directly corresponds to the character's 
/// ASCII value, enabling fast lookups without hash table lookups. Non-ASCII 
/// characters are assigned sequential IDs starting from a base value.
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
    /// Creates a new glyph with the specified symbol and pixel coordinates.
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

    /// Returns true if this glyph represents a single ASCII character.
    pub fn is_ascii(&self) -> bool {
        self.symbol.len() == 1
            && self.symbol.chars().next().unwrap().is_ascii()
    }
}


