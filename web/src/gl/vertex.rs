use std::slice;
use crate::error::Error;
use crate::gl::{Drawable, ShaderProgram, FontAtlas, GL, RenderContext};
use bon::bon;
use web_sys::{console, WebGl2RenderingContext};
use crate::bitmap_font::FontAtlasConfig;
use crate::gl::ubo::UniformBufferObject;
use crate::mat4::Mat4;
use crate::SimpleRng;

// todo: split vbo_instance into STATIC_DRAW vbo_cell(x, y) and STREAM vbo_data(depth, fg, bg)
pub struct TerminalGrid {
    /// Shader program for rendering the terminal cells.
    shader: ShaderProgram,
    /// Terminal cell instance data
    cells: Vec<TerminalCell>,
    /// shared state for the shader program
    ubo: UniformBufferObject,
    /// Vertex Array Object. Stores the per-vertex state of the model data.
    vao: web_sys::WebGlVertexArrayObject,
    /// Vertex Buffer Object. Stores the vertex data (model data).
    vbo: web_sys::WebGlBuffer,
    /// Instance Buffer Object. Stores the instance data (transform data).
    vbo_instance: web_sys::WebGlBuffer,
    /// Index Buffer Object. Stores the indices for the vertex data.
    index_buf: web_sys::WebGlBuffer,
    /// Font atlas for rendering text.
    atlas: FontAtlas,
    /// Uniform location for the texture sampler.
    sampler_loc: web_sys::WebGlUniformLocation,
}

#[bon]
impl TerminalGrid {
    const FRAGMENT_GLSL: &'static str = include_str!("../shaders/cell.frag");
    const VERTEX_GLSL: &'static str = include_str!("../shaders/cell.vert");

    // locations set in vertex shader
    const POS_ATTRIB: u32 = 0;
    const UV_ATTRIB: u32 = 1;

    #[builder]
    pub fn new(
        gl: &WebGl2RenderingContext,
        atlas: FontAtlas,
        font_config: &FontAtlasConfig,
        // transform_data: &[TerminalCell],
        screen_size: (i32, i32),
        // indices: &[u8],
    ) -> Result<Self, Error> {
        let (w, h) = (font_config.cell_width as f32, font_config.cell_height as f32);
        let model_data: [f32; 16] = [
            //  x      y     u     v
            w,   0.0,  1.0,  0.0,  // top-right
            0.0,   h,  0.0,  1.0,  // bottom-left
            w,     h,  1.0,  1.0,  // bottom-right
            0.0, 0.0,  0.0,  0.0,  // top-left
        ];
        
        let shader = ShaderProgram::create(gl, Self::VERTEX_GLSL, Self::FRAGMENT_GLSL)?;
        
        // create Vertex Array Object for storing the 
        let vao = gl.create_vertex_array()
            .ok_or(Error::VertexArrayCreationError)?;
        gl.bind_vertex_array(Some(&vao));

        // create and bind VBO
        let vbo = gl.create_buffer()
            .ok_or(Error::BufferCreationError("vbo"))?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        unsafe {
            let view = js_sys::Float32Array::view(&model_data);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // setup vertex attributes (while VAO is bound)
        const STRIDE: i32 = (2 + 2) * 4; // 4 floats per vertex
        
        // vertex shader in_* vars    attribute  count   type  offset
        enable_vertex_attrib(gl, Self::POS_ATTRIB, 2, GL::FLOAT, 0, STRIDE);
        enable_vertex_attrib(gl, Self::UV_ATTRIB,  2, GL::FLOAT, 8, STRIDE);

        // create and bind instance buffer
        let instance_buf = gl.create_buffer()
            .ok_or(Error::BufferCreationError("instance buffer"))?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&instance_buf));

        // upload instance data
        let cell_data = create_terminal_cell_data(screen_size, font_config);
        unsafe {
            let data_ptr = cell_data.as_ptr() as *const u8;
            let size = cell_data.len() * size_of::<TerminalCell>();
            let view = js_sys::Uint8Array::view(slice::from_raw_parts(data_ptr, size));
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        // setup instance attributes (while VAO is bound)
        use TerminalCell as ID;
        // vertex shader in_* vars        attribute    count      type         offset
        enable_vertex_attrib_array(gl, ID::POS_ATTRIB,   2, GL::UNSIGNED_SHORT,  0);
        enable_vertex_attrib_array(gl, ID::DEPTH_ATTRIB, 1, GL::FLOAT,           4);
        enable_vertex_attrib_array(gl, ID::FG_ATTRIB,    1, GL::UNSIGNED_INT,    8);
        enable_vertex_attrib_array(gl, ID::BG_ATTRIB,    1, GL::UNSIGNED_INT,    12);

        // create and bind index buffer (still part of VAO state)
        let index_buf = gl.create_buffer()
            .ok_or(Error::BufferCreationError("index buffer"))?;
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buf));

        // upload index data
        let indices = [
            0, 1, 2, // first triangle
            0, 3, 1, // second triangle
        ];
        gl.buffer_data_with_u8_array(GL::ELEMENT_ARRAY_BUFFER, &indices, GL::STATIC_DRAW);

        // unbind VAO to prevent accidental modification
        gl.bind_vertex_array(None);

        // setup uniform data
        let ubo = UniformBufferObject::new(gl, CellUbo::BINDING_POINT)?;
        ubo.bind_to_shader(gl, &shader, "CellUniforms")?;
        
        let sampler_loc = gl.get_uniform_location(&shader.program, "u_sampler")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_sampler"))?;

        console::log_2(&"terminal cells".into(), &cell_data.len().into());

        Ok(Self {
            shader,
            cells: cell_data,
            ubo,
            vao,
            vbo,
            index_buf,
            vbo_instance: instance_buf,
            atlas,
            sampler_loc,
            // projection_loc,
        })
    }
    
    pub(crate) fn upload_ubo_data(&self, gl: &WebGl2RenderingContext, data: CellUbo) {
        self.ubo.upload_data(gl, &data);
    }
}

fn enable_vertex_attrib_array(
    gl: &WebGl2RenderingContext,
    index: u32,
    size: i32,
    type_: u32,
    offset: i32,
) {
    enable_vertex_attrib(gl, index, size, type_, offset, size_of::<TerminalCell>() as i32);
    gl.vertex_attrib_divisor(index, 1);
}

fn enable_vertex_attrib(
    gl: &WebGl2RenderingContext,
    index: u32,
    size: i32,
    type_: u32,
    offset: i32,
    stride: i32,
) {
    gl.enable_vertex_attrib_array(index);
    if type_ == GL::FLOAT {
        gl.vertex_attrib_pointer_with_i32(index, size, type_, false, stride, offset);
    } else {
        gl.vertex_attrib_i_pointer_with_i32(index, size, type_, stride, offset);
    }
}


impl Drawable for TerminalGrid {
    fn prepare(&self, context: &mut RenderContext) {
        let gl = context.gl;
        
        self.shader.use_program(gl);
        
        gl.bind_vertex_array(Some(&self.vao));

        self.atlas.bind(gl, 0);
        self.ubo.bind(context.gl);
        gl.uniform1i(Some(&self.sampler_loc), 0);
    }

    fn draw(&self, context: &mut RenderContext) {
        let gl = context.gl;
        let cell_count = self.cells.len() as i32;
        gl.draw_elements_instanced_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_BYTE, 0, cell_count);
    }

    fn cleanup(&self, context: &mut RenderContext) {
        let gl = context.gl;
        gl.bind_vertex_array(None);
        gl.bind_texture(GL::TEXTURE_2D, None); // todo confirm: not TEXTURE_2D_ARRAY?

        self.ubo.unbind(gl)
    }
}



// todo: split into 2 structs
#[repr(C, align(4))]
pub(crate) struct TerminalCell {
    pub position: [u16; 2],
    pub depth: f32,
    pub fg: u32,
    pub bg: u32,
}

impl TerminalCell {
    pub(crate) const POS_ATTRIB: u32 = 2;
    pub(crate) const DEPTH_ATTRIB: u32 = 3;
    pub(crate) const FG_ATTRIB: u32 = 4;
    pub(crate) const BG_ATTRIB: u32 = 5;

    pub(crate) fn new(xy: (u16, u16), depth: u16, fg: u32, bg: u32) -> Self {
        Self { position: [xy.0, xy.1], depth: depth as f32, fg, bg }
    }
}


#[repr(C, align(16))] // std140 layout requires proper alignment
pub struct CellUbo {
    pub projection: [f32; 16], // mat4
    pub cell_size: [f32; 2],   // vec2
}

impl CellUbo {
    pub const BINDING_POINT: u32 = 0;

    pub fn new(projection: &Mat4, cell_width: i32, cell_height: i32) -> Self {
        Self {
            projection: projection.data,
            cell_size: [cell_width as f32, cell_height as f32],
        }
    }
}

fn create_terminal_cell_data(
    screen_size: (i32, i32),
    font_config: &FontAtlasConfig,
) -> Vec<TerminalCell> {
    let (cell_width, cell_height) = (font_config.cell_width, font_config.cell_height);
    let (cols, rows) = (screen_size.0 / cell_width, screen_size.1 / cell_height);

    let mut cells = Vec::new();

    let mut rng = SimpleRng::default();


    for row in 0..rows {
        for col in 0..cols {
            // let depth = (row * cols + col) % metadata.char_to_uv.len() as i32;
            // let (a, b) = ((col as usize) % s.len(), (col as usize + 1) % s.len());
            // let (a, b) = if a > b {
            //     (0, 1)
            // } else {
            //     (a, b)
            // };

            let fg = rng.gen() | 0xff;
            let bg = rng.gen() | 0xff;
            let fg = 0xffffffff;
            // let bg = 0x000000ff;
            let depth = 0;
            cells.push(TerminalCell::new((col as u16, row as u16), depth as u16, fg, bg));
        }
    }

    cells
}
