use std::slice;
use crate::error::Error;
use crate::gl::{Drawable, ShaderProgram, FontAtlas, GL};
use bon::bon;
use web_sys::{console, WebGl2RenderingContext};

pub struct CellArray {
    vao: web_sys::WebGlVertexArrayObject,
    vbo: web_sys::WebGlBuffer,
    instance_buf: web_sys::WebGlBuffer,
    index_buf: web_sys::WebGlBuffer,
    atlas: FontAtlas,
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

    #[builder]
    pub fn new(
        gl: &WebGl2RenderingContext,
        atlas: FontAtlas,
        model_data: &[f32],
        transform_data: &[InstanceData],
        indices: &[u8],
        shader: &ShaderProgram,
    ) -> Result<Self, Error> {
        // create VAO
        let vao = gl.create_vertex_array()
            .ok_or(Error::VertexArrayCreationError)?;
        gl.bind_vertex_array(Some(&vao));

        // create and bind VBO
        let vbo = gl.create_buffer()
            .ok_or(Error::BufferCreationError("vbo"))?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        // upload vertex data to GPU
        unsafe {
            let view = js_sys::Float32Array::view(model_data);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // setup vertex attributes (while VAO is bound)
        const STRIDE: i32 = (2 + 2) * 4; // 2+2 f32 for position and UV
        gl.vertex_attrib_pointer_with_i32(Self::POS_ATTRIB, 2, GL::FLOAT, false, STRIDE, 0);
        gl.enable_vertex_attrib_array(Self::POS_ATTRIB);
        gl.vertex_attrib_pointer_with_i32(Self::UV_ATTRIB, 2, GL::FLOAT, false, STRIDE, 2 * 4);
        gl.enable_vertex_attrib_array(Self::UV_ATTRIB);

        // create and bind instance buffer
        let instance_buf = gl.create_buffer()
            .ok_or(Error::BufferCreationError("instance buffer"))?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&instance_buf));

        // upload instance data
        unsafe {
            let data_ptr = transform_data.as_ptr() as *const u8;
            let size = transform_data.len() * size_of::<InstanceData>();
            let view = js_sys::Uint8Array::view(slice::from_raw_parts(data_ptr, size));
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // setup instance attributes (while VAO is bound)
        use InstanceData as ID;
        enable_vertex_attrib_array(gl, ID::POS_ATTRIB,   2, GL::UNSIGNED_SHORT,  0);
        enable_vertex_attrib_array(gl, ID::DEPTH_ATTRIB, 1, GL::FLOAT,           4);
        enable_vertex_attrib_array(gl, ID::FG_ATTRIB,    1, GL::UNSIGNED_INT,    8);
        enable_vertex_attrib_array(gl, ID::BG_ATTRIB,    1, GL::UNSIGNED_INT,    12);

        // create and bind index buffer (still part of VAO state)
        let index_buf = gl.create_buffer()
            .ok_or(Error::BufferCreationError("index buffer"))?;
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buf));

        // upload index data
        unsafe {
            let view = js_sys::Uint8Array::view(indices);
            gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // unbind VAO to prevent accidental modification
        gl.bind_vertex_array(None);

        // Get uniform locations
        let sampler_loc = gl.get_uniform_location(&shader.program, "u_sampler")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_sampler"))?;
        let projection_loc = gl.get_uniform_location(&shader.program, "u_projection")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_projection"))?;

        console::log_2(&"terminal cells".into(), &transform_data.len().into());

        Ok(Self {
            vao,
            vbo,
            index_buf,
            instance_buf,
            atlas,
            sampler_loc,
            projection_loc,
            count: transform_data.len() as i32,
        })
    }
}

fn enable_vertex_attrib_array(
    gl: &WebGl2RenderingContext,
    index: u32,
    size: i32,
    type_: u32,
    offset: i32,
) {
    const STRIDE: i32 = size_of::<InstanceData>() as i32;

    gl.enable_vertex_attrib_array(index);
    if type_ == GL::FLOAT {
        gl.vertex_attrib_pointer_with_i32(index, size, type_, false, STRIDE, offset);
    } else {
        gl.vertex_attrib_i_pointer_with_i32(index, size, type_, STRIDE, offset);
    }
    gl.vertex_attrib_divisor(index, 1);
}


impl Drawable for CellArray {
    fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_vertex_array(Some(&self.vao));

        self.atlas.bind(gl, 0);
        gl.uniform1i(Some(&self.sampler_loc), 0);
    }

    fn draw(&self, gl: &WebGl2RenderingContext) {
        gl.draw_elements_instanced_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_BYTE, 0, self.count);
    }

    fn unbind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_vertex_array(None);
        gl.bind_texture(GL::TEXTURE_2D, None);
    }
}


#[repr(C, align(4))]
pub(crate) struct InstanceData {
    pub position: [u16; 2],
    pub depth: f32,
    pub fg: u32,
    pub bg: u32,
}

impl InstanceData {
    pub(crate) const POS_ATTRIB: u32 = 2;
    pub(crate) const DEPTH_ATTRIB: u32 = 3;
    pub(crate) const FG_ATTRIB: u32 = 4;
    pub(crate) const BG_ATTRIB: u32 = 5;

    pub(crate) fn new(xy: (u16, u16), depth: u16, fg: u32, bg: u32) -> Self {
        Self { position: [xy.0, xy.1], depth: depth as f32, fg, bg }
    }
}
