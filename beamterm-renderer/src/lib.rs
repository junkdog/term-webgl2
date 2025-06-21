mod error;
mod gl;
mod mat4;
mod terminal;

pub(crate) mod js;

pub mod cell;
pub mod input;
#[cfg(feature = "js-api")]
pub mod wasm;

pub use ::beamterm_data::{FontAtlasData, GlyphEffect};
pub use beamterm_data::FontStyle;
pub use terminal::*;

pub use crate::{
    error::Error,
    gl::{CellData, FontAtlas, Renderer, TerminalGrid},
};

#[cfg(test)]
mod tests {
    use beamterm_data::FontAtlasData;

    #[test]
    fn test_font_atlas_config_deserialization() {
        let _ = FontAtlasData::default();
    }
}
