mod glyph;
mod serialization;
mod atlas;

pub use glyph::Glyph;
pub use atlas::FontAtlasConfig;
use serialization::*;

#[derive(Debug)]
pub struct FontAtlasDeserializationError {
    pub message: String,
}

