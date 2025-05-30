mod error;
mod mat4;
mod gl;

pub(crate) mod js;

pub use ::font_atlas::FontAtlasData;
pub use ::font_atlas::GlyphEffect;
pub use crate::error::Error;
pub use crate::gl::{CellData, FontAtlas, Renderer, TerminalGrid};
pub use mat4::Mat4;
pub use font_atlas::FontStyle;

pub const BITMAP_FONT_IMAGE: &[u8] = include_bytes!("../../data/bitmap_font.png");
pub const BITMAP_FONT_METADATA: &[u8] = include_bytes!("../../data/bitmap_font.atlas");

#[cfg(test)]
mod tests {
    use font_atlas::FontAtlasData;
    use crate::BITMAP_FONT_METADATA;

    #[test]
    fn test_font_atlas_config_deserialization() {
        let _ = FontAtlasData::from_binary(BITMAP_FONT_METADATA).unwrap();
    }
}