use web_sys::console;
use crate::error::Error;
use crate::gl::GL;

pub struct Texture {
    gl_texture: web_sys::WebGlTexture,
    width: i32,
    height: i32,
}

impl Texture {
    pub fn new(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        data: &[u8],
        width: i32,
        height: i32
    ) -> Result<Self, Error> {
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
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        
        Ok(Self { gl_texture, width, height })
    }

    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        // bind texture and set uniform
        gl.active_texture(GL::TEXTURE0 + texture_unit);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.gl_texture));
    }
}