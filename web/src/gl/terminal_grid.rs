use crate::font_atlas::FontAtlasConfig;
use crate::error::Error;
use crate::gl::ubo::UniformBufferObject;
use crate::gl::{Drawable, FontAtlas, RenderContext, ShaderProgram, GL};
use crate::mat4::Mat4;
use crate::SimpleRng;
use std::slice;
use web_sys::{console, WebGl2RenderingContext};

// todo: split vbo_instance into STATIC_DRAW vbo_cell(x, y) and STREAM vbo_data(depth, fg, bg)
pub struct TerminalGrid {
    /// Shader program for rendering the terminal cells.
    shader: ShaderProgram,
    /// Terminal cell instance data
    cells: Vec<TerminalCell>,
    /// Buffers for the terminal grid
    buffers: TerminalBuffers,
    /// shared state for the shader program
    ubo: UniformBufferObject,
    /// Vertex Array Object. Stores the per-vertex state of the model data.
    vao: web_sys::WebGlVertexArrayObject,
    /// Font atlas for rendering text.
    atlas: FontAtlas,
    /// Uniform location for the texture sampler.
    sampler_loc: web_sys::WebGlUniformLocation,
}

struct TerminalBuffers {
    vertices: web_sys::WebGlBuffer,
    instances: web_sys::WebGlBuffer,
    indices: web_sys::WebGlBuffer,
}


impl TerminalGrid {
    const FRAGMENT_GLSL: &'static str = include_str!("../shaders/cell.frag");
    const VERTEX_GLSL: &'static str = include_str!("../shaders/cell.vert");

    // locations set in vertex shader
    const POS_ATTRIB: u32 = 0;
    const UV_ATTRIB: u32 = 1;

    pub fn new(
        gl: &WebGl2RenderingContext,
        atlas: FontAtlas,
        screen_size: (i32, i32),
        cell_size: (i32, i32),
    ) -> Result<Self, Error> {
        // create and setup the Vertex Array Object
        let vao = create_vao(gl)?;
        gl.bind_vertex_array(Some(&vao));

        // prepare vertex, index and instance buffers
        let cell_data = create_terminal_cell_data(screen_size, cell_size);
        let buffers = setup_buffers(gl, &cell_data, cell_size)?;

        // unbind VAO to prevent accidental modification
        gl.bind_vertex_array(None);

        // setup shader and uniform data
        let shader = ShaderProgram::create(gl, Self::VERTEX_GLSL, Self::FRAGMENT_GLSL)?;
        shader.use_program(gl);
        
        let ubo = UniformBufferObject::new(gl, CellUbo::BINDING_POINT)?;
        ubo.bind_to_shader(gl, &shader, "CellUniforms")?;
        
        let sampler_loc = gl.get_uniform_location(&shader.program, "u_sampler")
            .ok_or(Error::UnableToRetrieveUniformLocation("u_sampler"))?;

        console::log_2(&"terminal cells".into(), &cell_data.len().into());

        let grid = Self {
            shader,
            cells: cell_data,
            buffers,
            ubo,
            vao,
            atlas,
            sampler_loc,
        }; 
        
        Ok(grid)
    }
    
    pub(crate) fn upload_ubo_data(
        &self, 
        gl: &WebGl2RenderingContext,
        screen_size: (i32, i32),
        cell_size: (i32, i32),
    ) {
        let data = CellUbo {
            projection: Mat4::orthographic_from_size(
                screen_size.0 as f32,
                screen_size.1 as f32
            ).data,
            cell_size: [cell_size.0 as f32, cell_size.1 as f32],
        };
        self.ubo.upload_data(gl, &data);
    }
}

fn create_vao(gl: &WebGl2RenderingContext) -> Result<web_sys::WebGlVertexArrayObject, Error> {
    gl.create_vertex_array()
        .ok_or(Error::VertexArrayCreationError)
}

fn setup_buffers(
    gl: &WebGl2RenderingContext,
    cell_data: &[TerminalCell],
    cell_size: (i32, i32),
) -> Result<TerminalBuffers, Error> {
    let (w, h) = (cell_size.0 as f32, cell_size.1 as f32);
    let vertices = [
        // x, y, u, v
          w, 0.0, 1.0, 0.0, // top-right
        0.0,   h, 0.0, 1.0, // bottom-left
          w,   h, 1.0, 1.0, // bottom-right
        0.0, 0.0, 0.0, 0.0  // top-left
    ];
    let indices = [0, 1, 2, 0, 3, 1];

    Ok(TerminalBuffers {
        vertices: create_buffer_f32(gl, GL::ARRAY_BUFFER, &vertices, GL::STATIC_DRAW)?,
        instances: create_buffer_for_instances(gl, cell_data)?,
        indices: create_buffer_u8(gl, GL::ELEMENT_ARRAY_BUFFER, &indices, GL::STATIC_DRAW)?,
    })
}

fn create_buffer_u8(
    gl: &WebGl2RenderingContext,
    target: u32,
    data: &[u8],
    usage: u32
) -> Result<web_sys::WebGlBuffer, Error> {
    let index_buf = gl.create_buffer()
        .ok_or(Error::BufferCreationError("vbo-u8"))?;
    gl.bind_buffer(target, Some(&index_buf));

    gl.buffer_data_with_u8_array(target, data, usage);
    
    Ok(index_buf)
}

fn create_buffer_f32(
    gl: &WebGl2RenderingContext,
    target: u32,
    data: &[f32],
    usage: u32
) -> Result<web_sys::WebGlBuffer, Error> {
    let buffer = gl.create_buffer()
        .ok_or(Error::BufferCreationError("vbo-f32"))?;

    gl.bind_buffer(target, Some(&buffer));

    unsafe {
        let view = js_sys::Float32Array::view(data);
        gl.buffer_data_with_array_buffer_view(target, &view, usage);
    }

    // vertex attributes \\
    const STRIDE: i32 = (2 + 2) * 4; // 4 floats per vertex
    enable_vertex_attrib(gl, TerminalGrid::POS_ATTRIB, 2, GL::FLOAT, 0, STRIDE);
    enable_vertex_attrib(gl, TerminalGrid::UV_ATTRIB,  2, GL::FLOAT, 8, STRIDE);

    Ok(buffer)
}


fn create_buffer_for_instances<T>(
    gl: &WebGl2RenderingContext,
    instance_data: &[T],
) -> Result<web_sys::WebGlBuffer, Error> {
    let instance_buf = gl.create_buffer()
        .ok_or(Error::BufferCreationError("instance-buffer"))?;
    
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&instance_buf));

    // upload instance data
    unsafe {
        let data_ptr = instance_data.as_ptr() as *const u8;
        let size = instance_data.len() * size_of::<T>();
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
    
    Ok(instance_buf)
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
struct TerminalCell {
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
struct CellUbo {
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
    cell_size: (i32, i32),
) -> Vec<TerminalCell> {
    let (cols, rows) = (screen_size.0 / cell_size.0, screen_size.1 / cell_size.1);

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
