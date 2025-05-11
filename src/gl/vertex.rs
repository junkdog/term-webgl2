use std::collections::HashMap;
use bon::{bon, builder};
use web_sys::{console, WebGl2RenderingContext};
use crate::error::Error;
use crate::gl::{Drawable, GL};
use crate::gl::texture::{debug_png, Texture};

pub struct CellArray {
    vbo: web_sys::WebGlBuffer,
    index_buf: web_sys::WebGlBuffer,
    // texture: web_sys::WebGlTexture,
    texture: Texture,
    sampler_loc: web_sys::WebGlUniformLocation,
    count: i32,
}

#[bon]
impl CellArray {
    pub const FRAGMENT_GLSL: &'static str = include_str!("../shaders/cell.frag");
    pub const VERTEX_GLSL: &'static str = include_str!("../shaders/cell.vert");

    // locations set in vertex shader
    const POS_ATTRIB: u32 = 0;
    const UV_ATTRIB: u32 = 1;

    const PIXELS: &'static [u8] = &[
        0x10,0,0,    0x20,0,0,    0x30,0,0,    0x40,0,0,
        0x50,0,0,    0x60,0,0,    0x70,0,0,    0x80,0,0,
        0x90,0,0,    0xA0,0,0,    0xB0,0,0,    0xC0,0,0,
        0xD0,0,0,    0xE0,0,0,    0xF0,0,0,    0,0,0x10,
        0,0,0x10,    0,0,0x20,    0,0,0x30,    0,0,0x40,
        0,0,0x50,    0,0,0x60,    0,0,0x70,    0,0,0x80,
    ];

    #[builder]
    pub fn new(
        gl: &WebGl2RenderingContext,
        vertices: &[f32],
        indices: &[u8],
        program: &web_sys::WebGlProgram,
    ) -> Result<Self, Error> {

        // create and bind VBO
        let vbo = gl.create_buffer()
            .ok_or(Error::BufferCreationError("vbo"))?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        let index_buf = gl.create_buffer()
            .ok_or(Error::BufferCreationError("index buffer"))?;
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buf));

        // upload vertex index data to GPU
        unsafe {
            let view = js_sys::Uint8Array::view(indices);
            gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // upload vertex data to GPU
        unsafe {
            let view = js_sys::Float32Array::view(vertices);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // get Sampler uniform location
        let sampler_loc = gl.get_uniform_location(program, "u_sampler")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_sampler"))?;

        // create texture
        const PIXELS: &[u8] = include_bytes!("../../data/bitmap_font.png");
        debug_png(PIXELS)?;
        // let texture = Texture::new(gl, GL::RGB, Self::PIXELS, 4, 6)?;
        let texture = Texture::from_image_data(gl, GL::RGBA, PIXELS)?;

        // let texture = gl.create_texture()
        //     .ok_or(Error::TextureCreationError)?;
        //
        // // upload texture data
        // gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        // unsafe {
        //     let view = js_sys::Uint8Array::view(Self::PIXELS);
        //     gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
        //         GL::TEXTURE_2D,
        //         0,
        //         GL::RGB as i32,
        //         4,
        //         6,
        //         0,
        //         GL::RGB,
        //         GL::UNSIGNED_BYTE,
        //         &view,
        //         0,
        //     ).map_err(|v| {
        //         console::error_2(&"Failed to upload texture data".into(), &v);
        //         Error::TextureCreationError
        //     })?;
        // }
        //
        // // setup and generate mipmap
        // gl.generate_mipmap(GL::TEXTURE_2D);
        // gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        // gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        // gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        // gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);

        Ok(Self {
            vbo,
            index_buf,
            texture,
            sampler_loc,
            count: indices.len() as i32,
        })
    }
}

impl Drawable for CellArray {
    fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buf));

        self.texture.bind(gl, 0);
        // gl.active_texture(GL::TEXTURE0 + 0);
        // gl.bind_texture(GL::TEXTURE_2D, Some(&self.texture));
    }

    fn draw(&self, gl: &WebGl2RenderingContext) {
        // set the sampler uniform to use texture unit 0
        gl.uniform1i(Some(&self.sampler_loc), 0);

        const STRIDE: i32 = (2 + 2) * 4; // 2+2 f32 for position and UV

        // set up vertex attribute pointer
        gl.vertex_attrib_pointer_with_i32(Self::POS_ATTRIB, 2, GL::FLOAT, false, STRIDE, 0);
        gl.enable_vertex_attrib_array(Self::POS_ATTRIB);

        // setup UV attribute pointer
        gl.vertex_attrib_pointer_with_i32(Self::UV_ATTRIB, 2, GL::FLOAT, false, STRIDE, 2 * 4);
        gl.enable_vertex_attrib_array(Self::UV_ATTRIB);

        // draw the elements
        gl.draw_elements_with_i32(GL::TRIANGLES, self.count, GL::UNSIGNED_BYTE, 0);
    }

    fn unbind(&self, gl: &WebGl2RenderingContext) {
        // unbind buffers to prevent accidental modification
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
        
        gl.bind_texture(GL::TEXTURE_2D, None);
    }
}