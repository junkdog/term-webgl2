mod error;
mod gl;
mod mat4;

pub(crate) mod js;

pub use ::beamterm_data::{FontAtlasData, GlyphEffect};
pub use beamterm_data::FontStyle;
pub use mat4::Mat4;

pub use crate::{
    error::Error,
    gl::{CellData, FontAtlas, Renderer, TerminalGrid},
};

pub const DEFAULT_FONT_ATLAS_BLOB: &[u8] = include_bytes!("../../data/bitmap_font.atlas");

#[cfg(test)]
mod tests {
    use beamterm_data::FontAtlasData;

    use crate::DEFAULT_FONT_ATLAS_BLOB;

    #[test]
    fn test_font_atlas_config_deserialization() {
        let _ = FontAtlasData::from_binary(DEFAULT_FONT_ATLAS_BLOB).unwrap();
    }
}
