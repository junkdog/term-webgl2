use bon::{bon, builder};
use web_sys::{console, WebGl2RenderingContext};
use crate::error::Error;

type GL = web_sys::WebGl2RenderingContext;

pub struct CellArray {
    vbo: web_sys::WebGlBuffer,
    index_buf: web_sys::WebGlBuffer,
    texture: web_sys::WebGlTexture,
    sampler_loc: web_sys::WebGlUniformLocation,
    count: i32,
}

pub struct IndexedVertexArray {
    vbo: web_sys::WebGlBuffer,
    index_buf: web_sys::WebGlBuffer,
    count: i32,
}

#[bon]
impl CellArray {
    pub const FRAGMENT_GLSL: &'static str = include_str!("../shaders/cell.frag");
    pub const VERTEX_GLSL: &'static str = include_str!("../shaders/cell.vert");

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

        // // set up vertex attribute pointer
        // gl.vertex_attrib_pointer_with_i32(
        //     Self::POS_ATTRIB,
        //     2,
        //     GL::FLOAT,
        //     false,
        //     (2 + 2) * 4,
        //     0,
        // );
        // gl.enable_vertex_attrib_array(Self::POS_ATTRIB);
        // 
        // // setup UV attribute pointer
        // gl.vertex_attrib_pointer_with_i32(
        //     Self::UV_ATTRIB,
        //     2,
        //     GL::FLOAT,
        //     false,
        //     (2 + 2) * 4,
        //     2 * 4,
        // );
        // gl.enable_vertex_attrib_array(Self::UV_ATTRIB);

        let texture = gl.create_texture()
            .ok_or(Error::TextureCreationError)?;

        let sampler_loc = gl.get_uniform_location(program, "u_sampler")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_sampler"))?;

        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        unsafe {
            let view = js_sys::Uint8Array::view(Self::PIXELS);
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
                GL::TEXTURE_2D,
                0,
                GL::RGB as i32,
                4,
                6,
                0,
                GL::RGB,
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

        Ok(Self {
            vbo,
            index_buf,
            texture,
            sampler_loc,
            count: indices.len() as i32,
        })
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext) {
        // bind buffers beofre drawing
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buf));

        // set up vertex attribute pointer
        gl.vertex_attrib_pointer_with_i32(
            Self::POS_ATTRIB,
            2,
            GL::FLOAT,
            false,
            (2 + 2) * 4,
            0,
        );
        gl.enable_vertex_attrib_array(Self::POS_ATTRIB);

        // setup UV attribute pointer
        gl.vertex_attrib_pointer_with_i32(
            Self::UV_ATTRIB,
            2,
            GL::FLOAT,
            false,
            (2 + 2) * 4,
            2 * 4,
        );
        gl.enable_vertex_attrib_array(Self::UV_ATTRIB);
        
        // bind texture and set uniform
        gl.active_texture(GL::TEXTURE0 + 1);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.texture));

        // Set the sampler uniform to use texture unit 1
        gl.uniform1i(Some(&self.sampler_loc), 1);

        // draw the elements
        gl.draw_elements_with_i32(GL::TRIANGLES, self.count, GL::UNSIGNED_BYTE, 0);

        // unbind buffers to prevent accidental modification
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
    }
}


#[bon]
impl IndexedVertexArray {
    #[builder]
    pub fn new(
        gl: &WebGl2RenderingContext,
        vertices: &[f32],
        indices: &[u8],
        attribute_location: u32,
        components_per_vertex: i32, // e.g., 2 for vec2, 3 for vec3
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

        // set up vertex attribute pointer
        gl.vertex_attrib_pointer_with_i32(
            attribute_location,
            components_per_vertex,
            GL::FLOAT,
            false,
            2 * 4,
            0,
        );
        gl.enable_vertex_attrib_array(attribute_location);

        Ok(Self {
            vbo,
            index_buf,
            count: indices.len() as i32,
        })
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext) {
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buf));

        gl.draw_elements_with_i32(GL::TRIANGLES, self.count, GL::UNSIGNED_BYTE, 0);

        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, None);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
    }
}

pub struct VertexArray {
    vao: web_sys::WebGlVertexArrayObject,
    count: i32,
}

#[bon]
impl VertexArray {
    #[builder]
    pub fn new(
        gl: &WebGl2RenderingContext,
        vertices: &[f32],
        attribute_location: u32,
        components_per_vertex: i32, // e.g., 2 for vec2, 3 for vec3
    ) -> Result<Self, Error> {
        // create and bind VAO
        let vao = gl.create_vertex_array()
            .ok_or(Error::VertexArrayCreationError)?;
        gl.bind_vertex_array(Some(&vao));

        // upload vertex data to GPU
        unsafe {
            let view = js_sys::Float32Array::view(vertices);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // set up vertex attribute pointer
        gl.vertex_attrib_pointer_with_i32(
            attribute_location,
            components_per_vertex,
            WebGl2RenderingContext::FLOAT,
            false,
            2 * 4,
            0,
        );
        gl.enable_vertex_attrib_array(attribute_location);

        // Unbind VAO to avoid accidental modification
        gl.bind_vertex_array(None);

        Ok(Self {
            vao,
            count: vertices.len() as i32 / components_per_vertex,
        })
    }

    pub fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_vertex_array(Some(&self.vao));
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext) {
        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, self.count);
    }
}