use crate::error::Error;
use crate::gl::GL;
use crate::BITMAP_FONT_IMAGE;
use compact_str::{CompactString, ToCompactString};
use font_atlas::FontAtlasConfig;
use image::{GenericImageView, ImageFormat};
use std::collections::HashMap;
use web_sys::console;

#[derive(Debug)]
pub(super) struct Texture {
    gl_texture: web_sys::WebGlTexture,
    pbo: web_sys::WebGlBuffer,
    pub(super) format: u32,
    width: i32,
    height: i32,
}

impl Texture {
    pub fn from_image_data(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        image_data: &[u8],
        metadata: &FontAtlasConfig,
    ) -> Result<Self, Error> {
        // load the image
        let img = image::load_from_memory_with_format(image_data, ImageFormat::Png)
            .map_err(|e| {
                console::error_1(&format!("Failed to load image: {:?}", e).into());
                Error::image_load_failed(&e.to_string())
            })?;
        
        // convert the image to RGBA format
        let (width, height) = img.dimensions();
        console::log_1(&format!("Image dimensions: {}x{}", width, height).into());
        
        let rgba_image = img.to_rgba8();
        let raw_data = rgba_image.as_raw();

        // create the texture
        Self::new(gl, format, raw_data, width as i32, height as i32, metadata)
    }

    pub fn new(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        data: &[u8],
        texture_width: i32,
        texture_height: i32,
        metadata: &FontAtlasConfig,
    ) -> Result<Self, Error> {
        console::log_1(&format!("Creating texture with format: {}", format).into());
        console::log_1(&format!("image={texture_width}x{texture_height}, grid={}x{}, glyps={}",
            metadata.cell_width, metadata.cell_height, metadata.glyphs.len()).into());
        console::log_1(&format!("Data length: {}kb", data.len() / 1024).into());

        let cell_width = metadata.cell_width;
        let cell_height = metadata.cell_height;

        // prepare texture
        let gl_texture = gl.create_texture()
            .ok_or(Error::texture_creation_failed())?;
        gl.bind_texture(GL::TEXTURE_2D_ARRAY, Some(&gl_texture));
        let texture_array_len = metadata.glyphs.iter().map(|g| g.id()).max().unwrap_or(0) + 1;
        gl.tex_storage_3d(GL::TEXTURE_2D_ARRAY, 1, GL::RGBA8, cell_width, cell_height, texture_array_len);

        // prepare a pbo for the the atlas, it will upload the texture data,
        // and then we will use gl.tex_sub_image_3d to upload the subregions
        let pbo = gl.create_buffer()
            .ok_or(Error::buffer_creation_failed("pbo"))?;

        gl.bind_buffer(GL::PIXEL_UNPACK_BUFFER, Some(&pbo));
        gl.buffer_data_with_u8_array(GL::PIXEL_UNPACK_BUFFER, data, GL::STATIC_DRAW);

        gl.pixel_storei(GL::UNPACK_ROW_LENGTH, texture_width);
        gl.pixel_storei(GL::UNPACK_IMAGE_HEIGHT, texture_height);

        Self::setup_mipmap(gl);

        Ok(Self { gl_texture, pbo, format, width: cell_width, height: cell_height })
    }

    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        // bind texture and set uniform
        gl.active_texture(GL::TEXTURE0 + texture_unit);
        gl.bind_texture(GL::TEXTURE_2D_ARRAY, Some(&self.gl_texture));
    }

    pub fn delete(&self, gl: &web_sys::WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.gl_texture));
        gl.delete_buffer(Some(&self.pbo));
    }

    pub fn gl_texture(&self) -> &web_sys::WebGlTexture {
        &self.gl_texture
    }

    fn setup_mipmap(gl: &web_sys::WebGl2RenderingContext) {
        gl.generate_mipmap(GL::TEXTURE_2D_ARRAY);
        gl.tex_parameteri(GL::TEXTURE_2D_ARRAY, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D_ARRAY, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D_ARRAY, GL::TEXTURE_BASE_LEVEL, 0);
        gl.tex_parameteri(GL::TEXTURE_2D_ARRAY, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D_ARRAY, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
    }
}
