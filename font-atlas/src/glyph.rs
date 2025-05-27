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
    /// The style of the glyph, e.g., bold, italic
    pub style: FontStyle,
    /// The character
    pub symbol: CompactString,
    /// The pixel coordinates of the glyph in the texture
    pub pixel_coords: (i32, i32),
    /// Indicates if the glyph is an emoji
    pub is_emoji: bool,
}

impl Glyph {
    /// The ID is used as a short-lived placeholder until the actual ID is assigned.
    pub const UNASSIGNED_ID: u16 = 0xFFFF;
    
    /// The ID bit for emojis, which is used to distinguish them from regular glyphs.
    pub const EMOJI_FLAG: u16 = 0x8000; // 0b1000000000000000
    pub const GLYPH_ID_MASK: u16 = FontStyle::Bold.id_mask() - 1; // 0x01FF
    

    /// Creates a new glyph with the specified symbol and pixel coordinates.
    pub fn new(symbol: &str, style: FontStyle, pixel_coords: (i32, i32)) -> Self {
        let first_char = symbol.chars().next().unwrap();
        let id = if symbol.len() == 1 && first_char.is_ascii() {
            // Use a different ID for non-ASCII characters
            first_char as u32 as u16
        } else {
            Self::UNASSIGNED_ID
        };

        Self {
            id,
            symbol: symbol.to_compact_string(),
            style,
            pixel_coords,
            is_emoji: false,
        }
    }
    
    pub fn new_emoji(symbol: &str, pixel_coords: (i32, i32)) -> Self {
        let id = Self::UNASSIGNED_ID; // Emojis are not assigned ASCII IDs
        Self {
            id,
            symbol: symbol.to_compact_string(),
            style: FontStyle::Normal,
            pixel_coords,
            is_emoji: true,
        }
    }

    /// Returns true if this glyph represents a single ASCII character.
    pub fn is_ascii(&self) -> bool {
        self.symbol.len() == 1
            && self.symbol.chars().next().unwrap().is_ascii()
    }
    
    pub fn id(&self) -> i32 {
        self.id as i32 | self.style as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal     = 0x0000,
    Bold       = 0x0200,
    Italic     = 0x0400,
    BoldItalic = 0x0600,
}

impl FontStyle {
    pub const ALL: [FontStyle; 4] = [
        FontStyle::Normal,
        FontStyle::Bold,
        FontStyle::Italic,
        FontStyle::BoldItalic,
    ];

    pub fn from_u8(v: u8) -> FontStyle {
        match v {
            0 => FontStyle::Normal,
            1 => FontStyle::Bold,
            2 => FontStyle::Italic,
            3 => FontStyle::BoldItalic,
            _ => panic!("Invalid font style value: {}", v),
        }
    }
    
    pub const fn ordinal(&self) -> usize {
        match self {
            FontStyle::Normal     => 0,
            FontStyle::Bold       => 1,
            FontStyle::Italic     => 2,
            FontStyle::BoldItalic => 3,
        }
    }
    
    /// Returns the style mask for this font style, used to encode the style in the glyph ID.
    pub const fn id_mask(&self) -> u16 {
        *self as u16
    }
    
    /// Returns the style mask for this font style, used to encode the style in the glyph ID.
    pub const fn layer_mask(&self) -> i32 {
        *self as i32
    }
}