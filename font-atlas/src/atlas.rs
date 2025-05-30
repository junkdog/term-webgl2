use std::fmt::Debug;
use crate::{Deserializer, FontAtlasDeserializationError, Glyph, Serializable};

#[derive(PartialEq)]
pub struct FontAtlasData {
    /// The font size in points
    pub font_size: f32,
    /// Width of the texture in pixels
    pub texture_width: u32,
    /// Height of the texture in pixels
    pub texture_height: u32,
    /// Depth of the texture in pixels
    pub texture_depth: u32,
    /// Width of each character cell
    pub cell_width: i32,
    /// Height of each character cell
    pub cell_height: i32,
    /// The glyphs in the font
    pub glyphs: Vec<Glyph>,
    /// The 3d texture data containing the font glyphs
    pub texture_data: Vec<u8>,
}

impl Debug for FontAtlasData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontAtlasData")
            .field("font_size", &self.font_size)
            .field("texture_width", &self.texture_width)
            .field("texture_height", &self.texture_height)
            .field("texture_depth", &self.texture_depth)
            .field("cell_width", &self.cell_width)
            .field("cell_height", &self.cell_height)
            .field("glyphs_count", &self.glyphs.len())
            .field("texture_data_kb", &(self.texture_data.len() * 4 / 1024))
            .finish()
    }
}


impl FontAtlasData {
    pub const PADDING: i32 = 1;
    pub const CELLS_PER_SLICE: i32 = 16;

    pub fn from_binary(serialized: &[u8]) -> Result<Self, FontAtlasDeserializationError> {
        let mut deserializer = Deserializer::new(serialized);
        FontAtlasData::deserialize(&mut deserializer)
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


impl Default for FontAtlasData {
    fn default() -> Self {
        Self::from_binary(include_bytes!("../../data/bitmap_font.atlas"))
            .unwrap()
    }
}
