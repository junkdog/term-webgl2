use std::collections::HashMap;
use compact_str::{CompactString, ToCompactString};
use web_sys::console;
use beamterm_data::{FontAtlasData, FontStyle};
use crate::error::Error;
use crate::gl::GL;

/// A texture atlas containing font glyphs for efficient WebGL text rendering.
///
/// `FontAtlas` manages a WebGL 2D texture array where each layer contains a single
/// character glyph. This design enables efficient instanced rendering of text by
/// allowing the GPU to select the appropriate character layer for each rendered cell.
///
/// # Architecture
/// The atlas uses a **WebGL 2D texture array** where:
/// - Each layer contains one character glyph
/// - ASCII characters use their ASCII value as the layer index
/// - Non-ASCII characters are stored in a hash map for layer lookup
/// - All glyphs have uniform cell dimensions for consistent spacing
#[derive(Debug)]
pub struct FontAtlas {
    /// The underlying texture
    texture: crate::gl::texture::Texture,
    /// Symbol to 3d texture index
    glyph_coords: HashMap<CompactString, u16>,
    /// The size of each character cell in pixels
    cell_size: (i32, i32),
    /// The number of slices in the atlas texture
    pub(super) num_slices: u32,
}


impl FontAtlas {
    /// Loads the default embedded font atlas.
    pub fn load_default(
        gl: &web_sys::WebGl2RenderingContext,
    ) -> Result<Self, Error> {
        let config = FontAtlasData::default();
        Self::load(gl, config)
    }

    /// Creates a TextureAtlas from a grid of equal-sized cells
    pub fn load(
        gl: &web_sys::WebGl2RenderingContext,
        config: FontAtlasData,
    ) -> Result<Self, Error> {
        let texture = crate::gl::texture::Texture::from_font_atlas_data(gl, GL::RGBA, &config)?;
        let num_slices = config.texture_layers;
        
        let texture_layers = config.glyphs.iter().map(|g| g.id as i32).max().unwrap_or(0) + 1;
        console::log_1(&format!("Creating atlas grid with {}/{texture_layers} layers",
            config.glyphs.len()).into());
        
        let (cell_width, cell_height) = (config.cell_width, config.cell_height);
        let mut layers = HashMap::new();

        // we only store the normal-styled glyphs (incl emoji) in the atlas lookup,
        // as the correct layer id can be derived from the base glyph id plus font style
        config.glyphs.iter()
            .filter(|g| g.style == FontStyle::Normal) // only normal style glyphs
            .filter(|g| !g.is_ascii())                // only non-ascii glyphs
            .for_each(|g| {
                layers.insert(g.symbol.to_compact_string(), g.id);
            });

        Ok(Self {
            texture,
            glyph_coords: layers,
            cell_size: (cell_width, cell_height),
            num_slices,
        })
    }

    /// Binds the atlas texture to the specified texture unit
    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        self.texture.bind(gl, texture_unit);
    }

    pub fn cell_size(&self) -> (i32, i32) {
        let (w, h) = self.cell_size;
        (w - 2 * FontAtlasData::PADDING, h - 2 * FontAtlasData::PADDING)
        // self.cell_size
    }

    /// Returns the texture array z-offset for the given key
    pub(super) fn get_glyph_coord(&self, key: &str, font_style: FontStyle) -> Option<u16> {
        if key.len() == 1 {
            let ch = key.chars().next().unwrap();
            if ch.is_ascii() { // 0x00..0x7f double as layer
                let id = ch as u16 | font_style.style_mask();
                return Some(id);
            }
        }

        self.glyph_coords.get(key)
            .copied()
            .map(|id| id | font_style.style_mask())
    }
}