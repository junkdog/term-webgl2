use crate::error::Error;
use crate::gl::GL;
use font_atlas::FontAtlasData;
use image::{GenericImageView, ImageFormat};
use web_sys::{console, WebGlBuffer};

#[derive(Debug)]
pub(super) struct Texture {
    gl_texture: web_sys::WebGlTexture,
    pub(super) format: u32,
    width: i32,
    height: i32,
}

impl Texture {
    // pub(super) fn from_font_atlas(
    //     gl: &web_sys::WebGl2RenderingContext,
    //     format: u32,
    //     metadata: &FontAtlasData,
    // ) -> Result<Self, Error> {
    //     // load the image
    //     // let img = image::load_from_memory_with_format(image_data, ImageFormat::Png)
    //     //     .map_err(|e| {
    //     //         console::error_1(&format!("Failed to load image: {:?}", e).into());
    //     //         Error::image_load_failed(&e.to_string())
    //     //     })?;
    //     
    //     // convert the image to RGBA format
    //     // let (width, height) = img.dimensions();
    //     // console::log_1(&format!("Image dimensions: {}x{}", width, height).into());
    //     // 
    //     // let rgba_image = img.to_rgba8();
    //     // let raw_data = rgba_image.as_raw();
    // 
    //     // create the texture
    //     Self::new(gl, format, metadata)
    // }

    pub(super) fn from_font_atlas_data(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        // texture_width: i32,
        // texture_height: i32,
        atlas: &FontAtlasData,
    ) -> Result<Self, Error> {
        console::log_1(&format!("Creating texture with format: {}", format).into());
        console::log_1(&format!("glyphs={}", atlas.glyphs.len()).into());
        console::log_1(&format!("3d texture layout {}x{}x{}",
            atlas.texture_width, atlas.texture_height, atlas.texture_depth).into());
        
        let data_len = atlas.texture_data.len() * 4;
        console::log_1(&format!("Texture data length: {}kb", data_len / 1024).into());

        let cell_width = atlas.cell_width;
        let cell_height = atlas.cell_height;

        let (width, height, depth) = (
            atlas.texture_width as i32,
            atlas.texture_height as i32,
            atlas.texture_depth as i32
        );
        
        // prepare texture
        let gl_texture = gl.create_texture()
            .ok_or(Error::texture_creation_failed())?;
        gl.bind_texture(GL::TEXTURE_3D, Some(&gl_texture));
        gl.tex_storage_3d(GL::TEXTURE_3D, 1, GL::RGBA8, width, height, depth);
        
        
        // upload the texture data; convert to u8 array
        let texture_data: Vec<u8> = atlas.texture_data.iter()
            .flat_map(|&color| color.to_le_bytes())
            .collect();

        gl.tex_sub_image_3d_with_opt_u8_array_and_src_offset(
            GL::TEXTURE_3D,
            0, // level
            0, 0, 0, // offset
            width, height, depth, // texture size
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            Some(&texture_data),
            0 // src offset
        ).map_err(|_| Error::texture_creation_failed())?;
        
        Self::setup_mipmap_3d(gl);

        Ok(Self { gl_texture, format, width: cell_width, height: cell_height })
    }

    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        // bind texture and set uniform
        gl.active_texture(GL::TEXTURE0 + texture_unit);
        gl.bind_texture(GL::TEXTURE_3D, Some(&self.gl_texture));
    }

    pub fn delete(&self, gl: &web_sys::WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.gl_texture));
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

    fn setup_mipmap_3d(gl: &web_sys::WebGl2RenderingContext) {
        gl.tex_parameteri(GL::TEXTURE_3D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_3D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_3D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_3D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_3D, GL::TEXTURE_WRAP_R, GL::CLAMP_TO_EDGE as i32);
    }
}
