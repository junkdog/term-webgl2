use std::collections::HashMap;
use serde::Deserialize;
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct BitmapFontMetadata {
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
    /// Mapping from characters to UV coordinates (u1, v1, u2, v2)
    pub char_to_uv: HashMap<char, (f32, f32, f32, f32)>,
    /// Mapping from characters to pixel coordinates (x, y)
    pub char_to_px: HashMap<char, (i32, i32)>,
}

impl BitmapFontMetadata {
    pub const PADDING: i32 = 1;
    
    pub fn from_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json)
            .map_err(|e| Error::JsonDeserializationError(format!("Failed to deserialize JSON: {}", e)))
    }

    pub fn terminal_size(
        &self,
        viewport_width: i32,
        viewport_height: i32
    ) -> (i32, i32) {
        (viewport_width / self.cell_width, viewport_height / self.cell_height)
    }
}
