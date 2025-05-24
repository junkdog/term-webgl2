use crate::{Deserializer, FontAtlasDeserializationError, Glyph, Serializable};

#[derive(Debug)]
pub struct FontAtlasConfig {
    /// The font size in points
    pub font_size: f32,
    /// Width of the texture in pixels
    pub texture_width: u32,
    /// Height of the texture in pixels
    pub texture_height: u32,
    /// Width of each character cell
    pub cell_width: i32,
    /// Height of each character cell
    pub cell_height: i32,
    /// The glyphs in the font
    pub glyphs: Vec<Glyph>,
}


impl FontAtlasConfig {
    pub const PADDING: i32 = 1;

    pub fn from_binary(serialized: &[u8]) -> Result<Self, FontAtlasDeserializationError> {
        let mut deserializer = Deserializer::new(serialized);
        FontAtlasConfig::deserialize(&mut deserializer)
            .map_err(|e| FontAtlasDeserializationError {
                message: format!("Failed to deserialize font atlas: {}", e.message),
            })
    }
    
    pub fn to_binary(&self) -> Vec<u8> {
        self.serialize()
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


impl Default for FontAtlasConfig {
    fn default() -> Self {
        Self::from_binary(include_bytes!("../../data/bitmap_font.atlas"))
            .unwrap()
    }
}
