use std::collections::HashMap;
use serde::Deserialize;

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
}