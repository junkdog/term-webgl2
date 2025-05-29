mod glyph;
mod serialization;
mod atlas;

pub use glyph::{Glyph, FontStyle};
pub use atlas::FontAtlasData;
use serialization::*;

#[derive(Debug)]
pub struct FontAtlasDeserializationError {
    pub message: String,
}

