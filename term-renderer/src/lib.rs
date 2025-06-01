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

pub const DEFAULT_FONT_ATLAS_BLOB: &[u8] = include_bytes!("../../data/bitmap_font.atlas");

#[cfg(test)]
mod tests {
    use font_atlas::FontAtlasData;
    use crate::DEFAULT_FONT_ATLAS_BLOB;

    #[test]
    fn test_font_atlas_config_deserialization() {
        let _ = FontAtlasData::from_binary(DEFAULT_FONT_ATLAS_BLOB).unwrap();
    }
}