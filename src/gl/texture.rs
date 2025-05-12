use std::collections::HashMap;
use image::{GenericImageView, ImageFormat};
use image::metadata::Orientation;
use web_sys::console;
use crate::error::Error;
use crate::gl::GL;

pub struct Texture {
    gl_texture: web_sys::WebGlTexture,
    width: i32,
    height: i32,
}

impl Texture {
    pub fn from_image_data(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        image_data: &[u8],
    ) -> Result<Self, Error> {
        // load the image
        let mut img = image::load_from_memory_with_format(image_data, ImageFormat::Png)
            .map_err(|e| {
                console::error_1(&format!("Failed to load image: {:?}", e).into());
                Error::ImageLoadError("failed to load image data")
            })?;
        
        img.apply_orientation(Orientation::FlipVertical);

        // convert the image to RGBA format
        let (width, height) = img.dimensions();
        console::log_1(&format!("Image dimensions: {}x{}", width, height).into());
        
        let rgba_image = img.to_rgba8();
        let raw_data = rgba_image.as_raw();

        // create the texture
        Self::new(gl, format, raw_data, width as i32, height as i32)
    }

    pub fn new(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        data: &[u8],
        width: i32,
        height: i32
    ) -> Result<Self, Error> {
        console::log_1(&format!("Creating texture with format: {}, width: {}, height: {}", format, width, height).into());
        console::log_1(&format!("Data length: {}", data.len()).into());

        // expected data length for error checking
        let expected_length = (width * height * 4) as usize; // 4 bytes per pixel for RGBA
        if data.len() != expected_length && format == GL::RGBA {
            console::warn_1(&format!("Data length mismatch: got {}, expected {}", data.len(), expected_length).into());
        }
        
        let gl_texture = gl.create_texture()
            .ok_or(Error::TextureCreationError)?;
        gl.bind_texture(GL::TEXTURE_2D, Some(&gl_texture));
        
        unsafe {
            let view = js_sys::Uint8Array::view(data);
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
                GL::TEXTURE_2D,
                0,
                format as i32,
                width,
                height,
                0,
                format,
                GL::UNSIGNED_BYTE,
                &view,
                0,
            ).map_err(|v| {
                console::error_2(&"Failed to upload texture data".into(), &v);
                Error::TextureCreationError
            })?;
        }

        gl.generate_mipmap(GL::TEXTURE_2D);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);

        Ok(Self { gl_texture, width, height })
    }

    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        // bind texture and set uniform
        gl.active_texture(GL::TEXTURE0 + texture_unit);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.gl_texture));
    }

    pub fn delete(&self, gl: &web_sys::WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.gl_texture));
    }

    pub fn gl_texture(&self) -> &web_sys::WebGlTexture {
        &self.gl_texture
    }
}

/// Represents a region within a texture atlas
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    /// UV coordinates (u1, v1, u2, v2)
    pub uvs: (f32, f32, f32, f32),
    /// Pixel coordinates (x, y)
    pub pos: (i32, i32),
    /// Width of the sprite in pixels
    pub width: i32,
    /// Height of the sprite in pixels
    pub height: i32,
}


/// A texture atlas that contains multiple sprites packed into a single texture
pub struct TextureAtlas {
    /// The underlying texture
    texture: Texture,
    /// Map of sprite names to their regions
    regions: HashMap<u16, AtlasRegion>,
}


impl TextureAtlas {
    /// Creates a new empty texture atlas with the specified dimensions
    pub fn new(
        texture: Texture,
    ) -> Result<Self, Error> {
        Ok(Self {
            texture,
            regions: HashMap::new(),
        })
    }

    /// Gets a region by name
    pub fn get_region(&self, key: u16) -> Option<&AtlasRegion> {
        self.regions.get(&key)
    }

    /// Binds the atlas texture to the specified texture unit
    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        self.texture.bind(gl, texture_unit);
    }

    /// Create vertices for rendering a sprite from this atlas
    pub fn create_sprite_vertices(
        &self,
        region: u16,
        x: f32,
        y: f32,
        width: f32,
        height: f32
    ) -> Option<[f32; 16]> {
        let region = self.get_region(region)?;
        let (u1, v1, u2, v2) = region.uvs;

        // Create a quad with position and texture coordinates
        let vertices = [
            // x, y, u, v
            x + width, y,          u2, v1,  // top-right
            x,         y + height, u1, v2,  // bottom-left
            x + width, y + height, u2, v2,  // bottom-right
            x,         y,          u1, v1,  // top-left
        ];

        Some(vertices)
    }

    /// Creates a TextureAtlas from a grid of equal-sized cells
    pub fn from_grid(
        texture: Texture,
    ) -> Result<Self, Error> {
        // 
        let cols: i32 = 50;
        let rows: i32 = 6;
        let cell_width: i32 = 18;
        let cell_height: i32 = 28;
        let padding: i32 = 1;
        
        let img_width = texture.width;
        let img_height = texture.height;

        console::log_1(&format!("Creating atlas grid: {}x{} cells", cols, rows).into());

        let mut regions = HashMap::new();

        // Create regions for each cell
        let mut idx = 0;
        for row in 0..rows {
            for col in 0..cols {
                let x = col * (cell_width + padding);
                let y = row * (cell_height + padding);

                // skip if this would go outside the image bounds
                if x + cell_width > img_width || y + cell_height > img_height {
                    continue;
                }

                // calculate UV coordinates
                let u1 = x as f32 / img_width as f32;
                let v1 = y as f32 / img_height as f32;
                let u2 = (x + cell_width) as f32 / img_width as f32;
                let v2 = (y + cell_height) as f32 / img_height as f32;

                // store the region with a generated name
                let region = AtlasRegion {
                    uvs: (u1, v1, u2, v2),
                    pos: (x, y),
                    width: cell_width,
                    height: cell_height,
                };
                regions.insert(idx, region);
                idx += 1;
            }
        }

        Ok(Self {
            texture,
            regions,
        })
    }
}