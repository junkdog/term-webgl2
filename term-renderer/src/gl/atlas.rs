use std::collections::HashMap;
use compact_str::{CompactString, ToCompactString};
use web_sys::console;
use font_atlas::{FontAtlasConfig, FontStyle};
use crate::BITMAP_FONT_IMAGE;
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
    /// Symbol to texture 2d array layer
    layers: HashMap<CompactString, i32>,
    /// The size of each character cell in pixels
    cell_size: (i32, i32),
}


impl FontAtlas {
    /// Loads the default embedded font atlas.
    pub fn load_default(
        gl: &web_sys::WebGl2RenderingContext,
    ) -> Result<Self, Error> {
        let config = FontAtlasConfig::default();
        Self::load(gl, BITMAP_FONT_IMAGE, config)
    }

    /// Creates a TextureAtlas from a grid of equal-sized cells
    pub fn load(
        gl: &web_sys::WebGl2RenderingContext,
        texture_data: &[u8],
        config: FontAtlasConfig,
    ) -> Result<Self, Error> {
        console::log_1(&format!("loading texture, {} bytes", texture_data.len()).into());

        let pbo = gl.create_buffer()
            .ok_or(Error::buffer_creation_failed("pbo"))?;
        gl.bind_buffer(GL::PIXEL_UNPACK_BUFFER, Some(&pbo));
        
        let texture = crate::gl::texture::Texture::from_image_data(gl, GL::RGBA, Some(&pbo), texture_data, &config)?;


        let texture_layers = config.glyphs.iter().map(|g| g.id()).max().unwrap_or(0) + 1;
        console::log_1(&format!("Creating atlas grid with {}/{texture_layers} layers", config.glyphs.len()).into());
        let (cell_width, cell_height) = (config.cell_width, config.cell_height);
        let mut layers = HashMap::new();

        for glyph in config.glyphs.iter() {
            gl.pixel_storei(GL::UNPACK_SKIP_PIXELS, glyph.pixel_coords.0);
            gl.pixel_storei(GL::UNPACK_SKIP_ROWS, glyph.pixel_coords.1);

            gl.tex_sub_image_3d_with_i32(
                GL::TEXTURE_2D_ARRAY,
                0,
                FontAtlasConfig::PADDING,
                FontAtlasConfig::PADDING,
                glyph.id(),
                cell_width - FontAtlasConfig::PADDING * 2,
                cell_height - FontAtlasConfig::PADDING * 2,
                1, // only one layer
                texture.format,
                GL::UNSIGNED_BYTE,
                0, // use pbo
            ).map_err(|v| {
                console::error_2(&"Failed to define subregion for ".into(), &v);
                Error::texture_creation_failed()
            })?;

            // we only store the normal-styled glyphs in the atlas lookup, as the correct
            // layer id can be derived from the base glyph id plus the font style
            if glyph.style != FontStyle::Normal {
                continue;
            }

            // ascii characters do not require a lookup table
            if !glyph.is_ascii() {
                layers.insert(glyph.symbol.to_compact_string(), glyph.id as i32);
            }
        }

        gl.bind_buffer(GL::PIXEL_UNPACK_BUFFER, None);
        gl.delete_buffer(Some(&pbo));

        Ok(Self {
            texture,
            layers,
            cell_size: (cell_width, cell_height),
        })
    }

    /// Binds the atlas texture to the specified texture unit
    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        self.texture.bind(gl, texture_unit);
    }

    pub fn cell_size(&self) -> (i32, i32) {
        self.cell_size
    }

    /// Returns the texture array z-offset for the given key
    pub(super) fn get_glyph_layer(&self, key: &str, font_style: FontStyle) -> Option<i32> {
        if key.len() == 1 {
            let ch = key.chars().next().unwrap();
            if ch.is_ascii() { // 0x00..0xff double as layer
                let id = ch as i32 | font_style.layer_mask();
                return Some(id);
            }
        }

        self.layers.get(key)
            .copied()
            .map(|id| id | font_style.layer_mask())
    }
}