use crate::error::Error;
use crate::gl::texture::Texture;
use crate::gl::{Drawable, ShaderProgram, TextureAtlas, GL};
use bon::bon;
use web_sys::WebGl2RenderingContext;

pub struct CellArray {
    vbo: web_sys::WebGlBuffer,
    index_buf: web_sys::WebGlBuffer,
    atlas: TextureAtlas,
    sampler_loc: web_sys::WebGlUniformLocation,
    projection_loc: web_sys::WebGlUniformLocation,
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
        atlas: TextureAtlas,
        vertices: &[f32],
        indices: &[u8],
        shader: &ShaderProgram,
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
        let sampler_loc = gl.get_uniform_location(&shader.program, "u_sampler")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_sampler"))?;

        // get projection uniform location
        let projection_loc = gl.get_uniform_location(&shader.program, "u_projection")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_projection"))?;
        
        Ok(Self {
            vbo,
            index_buf,
            atlas,
            sampler_loc,
            projection_loc,
            count: indices.len() as i32,
        })
    }
}

impl Drawable for CellArray {
    fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buf));

        // bind texture and set the sampler uniform to use texture unit 0
        self.atlas.bind(gl, 0);
        gl.uniform1i(Some(&self.sampler_loc), 0);

        const STRIDE: i32 = (2 + 2) * 4; // 2+2 f32 for position and UV

        // set up vertex attribute pointer
        gl.vertex_attrib_pointer_with_i32(Self::POS_ATTRIB, 2, GL::FLOAT, false, STRIDE, 0);
        gl.enable_vertex_attrib_array(Self::POS_ATTRIB);

        // setup UV attribute pointer
        gl.vertex_attrib_pointer_with_i32(Self::UV_ATTRIB, 2, GL::FLOAT, false, STRIDE, 2 * 4);
        gl.enable_vertex_attrib_array(Self::UV_ATTRIB);
    }

    fn draw(&self, gl: &WebGl2RenderingContext) {
        gl.draw_elements_with_i32(GL::TRIANGLES, self.count, GL::UNSIGNED_BYTE, 0);
    }

    fn unbind(&self, gl: &WebGl2RenderingContext) {
        // unbind buffers to prevent accidental modification
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
        
        gl.bind_texture(GL::TEXTURE_2D, None);
    }
}