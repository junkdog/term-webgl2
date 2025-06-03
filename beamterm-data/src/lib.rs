mod glyph;
mod serialization;
mod atlas;

pub use glyph::{Glyph, GlyphEffect, FontStyle};
pub use atlas::FontAtlasData;
use serialization::*;

#[derive(Debug)]
pub struct FontAtlasDeserializationError {
    pub message: String,
}

