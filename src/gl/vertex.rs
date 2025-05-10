use bon::{bon, builder};
use web_sys::WebGl2RenderingContext;
use crate::error::Error;

pub struct VertexArray {
    vao: web_sys::WebGlVertexArrayObject,
    vbo: web_sys::WebGlBuffer,
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

        // create and bind VBO
        let vbo = gl.create_buffer()
            .ok_or(Error::BufferCreationError)?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        // upload data to GPU
        unsafe {
            let array_view = js_sys::Float32Array::view(vertices);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &array_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
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
            vbo,
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