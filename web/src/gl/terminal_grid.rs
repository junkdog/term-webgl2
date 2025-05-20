use crate::error::Error;
use crate::gl::ubo::UniformBufferObject;
use crate::gl::{buffer_upload_array, buffer_upload_struct, Drawable, FontAtlas, RenderContext, ShaderProgram, GL};
use crate::mat4::Mat4;
use std::slice;
use web_sys::{console, WebGl2RenderingContext};
use crate::gl::vbo::VertexBufferObject;

// todo: split vbo_instance into STATIC_DRAW vbo_cell(x, y) and STREAM vbo_data(depth, fg, bg)
pub struct TerminalGrid {
    /// Shader program for rendering the terminal cells.
    shader: ShaderProgram,
    /// Terminal cell instance data
    cells: Vec<CellDynamic>,
    /// Terminal size in cells
    terminal_size: (u16, u16),
    /// Buffers for the terminal grid
    buffers: TerminalBuffers,
    /// shared state for the shader program
    ubo: UniformBufferObject,
    /// Font atlas for rendering text.
    atlas: FontAtlas,
    /// Uniform location for the texture sampler.
    sampler_loc: web_sys::WebGlUniformLocation,
}

struct TerminalBuffers {
    vao: web_sys::WebGlVertexArrayObject,
    vertices: web_sys::WebGlBuffer,
    instance_pos: web_sys::WebGlBuffer,
    instance_cell: web_sys::WebGlBuffer,
    indices: web_sys::WebGlBuffer,
}

impl TerminalBuffers {
    fn upload_instance_data<T>(
        &self,
        gl: &WebGl2RenderingContext,
        cell_data: &[T],
    ) {
        gl.bind_vertex_array(Some(&self.vao));

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.instance_cell));
        buffer_upload_array(gl, GL::ARRAY_BUFFER, cell_data, GL::DYNAMIC_DRAW);

        gl.bind_vertex_array(None);
    }
}


impl TerminalGrid {
    const FRAGMENT_GLSL: &'static str = include_str!("../shaders/cell.frag");
    const VERTEX_GLSL: &'static str = include_str!("../shaders/cell.vert");

    pub fn new(
        gl: &WebGl2RenderingContext,
        atlas: FontAtlas,
        screen_size: (i32, i32),
    ) -> Result<Self, Error> {
        // create and setup the Vertex Array Object
        let vao = create_vao(gl)?;
        gl.bind_vertex_array(Some(&vao));

        // prepare vertex, index and instance buffers
        let cell_size = atlas.cell_size();
        let (cols, rows) = (screen_size.0 / cell_size.0, screen_size.1 / cell_size.1);
        let cell_data = create_terminal_cell_data(cols, rows);
        let cell_pos = CellStatic::create_grid(cols, rows);
        let buffers = setup_buffers(gl, vao, &cell_pos, &cell_data, cell_size)?;

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
        
        let (cols, rows) = (screen_size.0 / cell_size.0, screen_size.1 / cell_size.1);
        console::log_1(&format!("terminal size {cols}x{rows}").into());
        let grid = Self {
            shader,
            terminal_size: (cols as u16, rows as u16),
            cells: cell_data,
            buffers,
            ubo,
            atlas,
            sampler_loc,
        };

        Ok(grid)
    }

    pub fn cell_size(&self) -> (i32, i32) {
        self.atlas.cell_size()
    }
    
    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal_size
    }

    pub fn upload_ubo_data(
        &self,
        gl: &WebGl2RenderingContext,
        screen_size: (i32, i32),
        cell_size: (i32, i32),
    ) {
        // todo: this should reflect on self.cell_size - but needs more wörk
        let data = CellUbo {
            projection: Mat4::orthographic_from_size(
                screen_size.0 as f32,
                screen_size.1 as f32
            ).data,
            cell_size: [cell_size.0 as f32, cell_size.1 as f32],
        };
        console::log_1(&format!("cell size: {:?}", data.cell_size).into());
        console::log_1(&format!("screen size: {:?}", screen_size).into());
        self.ubo.upload_data(gl, &data);
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn update_cells<'a>(
        &mut self,
        gl: &WebGl2RenderingContext,
        cells: impl Iterator<Item = CellData<'a>>,
    ) -> Result<(), Error> {
        // update instance buffer with new cell data
        let atlas = &self.atlas;

        let cells = cells.collect::<Vec<_>>();
        assert_eq!(cells.len(), self.cells.len());

        self.cells.iter_mut()
            .zip(cells)
            .for_each(|(cell, data)| {
                cell.fg = data.fg;
                cell.bg = data.bg;
                cell.depth = atlas.get_glyph_depth(data.symbol).map(|d| d as f32).unwrap_or(0.0);
            });

        gl.bind_vertex_array(Some(&self.buffers.vao));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffers.instance_cell));
        self.buffers.upload_instance_data(gl, &self.cells);
        gl.bind_vertex_array(None);

        Ok(())
    }
}

fn create_vao(gl: &WebGl2RenderingContext) -> Result<web_sys::WebGlVertexArrayObject, Error> {
    gl.create_vertex_array()
        .ok_or(Error::VertexArrayCreationError)
}

fn setup_buffers(
    gl: &WebGl2RenderingContext,
    vao: web_sys::WebGlVertexArrayObject,
    cell_pos: &[CellStatic],
    cell_data: &[CellDynamic],
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
        vao,
        vertices: create_buffer_f32(gl, GL::ARRAY_BUFFER, &vertices, GL::STATIC_DRAW)?,
        // instances: create_buffer_for_instances(gl, cell_data)?,
        instance_pos: create_static_instance_buffer(gl, cell_pos)?,
        instance_cell: create_dynamic_instance_buffer(gl, cell_data)?,
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
    enable_vertex_attrib(gl, attrib::POS, 2, GL::FLOAT, 0, STRIDE);
    enable_vertex_attrib(gl, attrib::UV,  2, GL::FLOAT, 8, STRIDE);

    Ok(buffer)
}


fn create_static_instance_buffer(
    gl: &WebGl2RenderingContext,
    instance_data: &[CellStatic],
) -> Result<web_sys::WebGlBuffer, Error> {
    let instance_buf = gl.create_buffer()
        .ok_or(Error::BufferCreationError("static-instance-buffer"))?;


    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&instance_buf));
    buffer_upload_array(gl, GL::ARRAY_BUFFER, instance_data, GL::STATIC_DRAW);

    let stride = size_of::<CellStatic>() as i32;
    enable_vertex_attrib_array(gl, attrib::GRID_XY, 2, GL::UNSIGNED_SHORT, 0, stride);

    Ok(instance_buf)
}

fn create_dynamic_instance_buffer(
    gl: &WebGl2RenderingContext,
    instance_data: &[CellDynamic],
) -> Result<web_sys::WebGlBuffer, Error> {
    let instance_buf = gl.create_buffer()
        .ok_or(Error::BufferCreationError("dynamic-instance-buffer"))?;
    
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&instance_buf));
    buffer_upload_array(gl, GL::ARRAY_BUFFER, instance_data, GL::DYNAMIC_DRAW);
    
    let stride = size_of::<CellDynamic>() as i32;

    // setup instance attributes (while VAO is bound)
    enable_vertex_attrib_array(gl, attrib::DEPTH, 1, GL::FLOAT,           0, stride);
    enable_vertex_attrib_array(gl, attrib::FG,    1, GL::UNSIGNED_INT,    4, stride);
    enable_vertex_attrib_array(gl, attrib::BG,    1, GL::UNSIGNED_INT,    8, stride);

    Ok(instance_buf)
}

fn enable_vertex_attrib_array(
    gl: &WebGl2RenderingContext,
    index: u32,
    size: i32,
    type_: u32,
    offset: i32,
    stride: i32,
) {
    enable_vertex_attrib(gl, index, size, type_, offset, stride);
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

        gl.bind_vertex_array(Some(&self.buffers.vao));

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
        gl.bind_texture(GL::TEXTURE_2D_ARRAY, None);

        self.ubo.unbind(gl)
    }
}

#[derive(Debug)]
pub struct CellData<'a> {
    pub symbol: &'a str,
    pub fg: u32,
    pub bg: u32,
}

impl<'a> CellData<'a> {
    pub fn new(symbol: &'a str, fg: u32, bg: u32) -> Self {
        Self { symbol, fg, bg }
    }
}

#[repr(C, align(4))]
struct CellStatic {
    pub grid_xy: [u16; 2],
}

#[repr(C, align(4))]
struct CellDynamic {
    // pub data: [u8; 8], // 2b depth, fg:rgb, bg:rgb

    pub depth: f32,
    pub fg: u32,
    pub bg: u32,
}

impl CellStatic {
    pub(crate) const POS_ATTRIB: u32 = 2;

    fn create_grid(cols: i32, rows: i32) -> Vec<Self> {
        debug_assert!(cols > 0 && cols < u16::MAX as i32, "cols: {cols}");
        debug_assert!(rows > 0 && rows < u16::MAX as i32, "rows: {rows}");
        
        (0..rows)
            .flat_map(|row| (0..cols).map(move |col| (col, row)))
            .map(|(col, row)| Self { grid_xy: [col as u16, row as u16] })
            .collect()
    }
}

impl CellDynamic {
    pub(crate) const DEPTH_ATTRIB: u32 = 3;
    pub(crate) const FG_ATTRIB: u32 = 4;
    pub(crate) const BG_ATTRIB: u32 = 5;

    pub(crate) fn new(depth: u16, fg: u32, bg: u32) -> Self {
        Self { depth: depth as f32, fg, bg }
    }
}


#[repr(C, align(16))] // std140 layout requires proper alignment
struct CellUbo {
    pub projection: [f32; 16], // mat4
    pub cell_size: [f32; 2],   // vec2
}

impl CellUbo {
    pub const BINDING_POINT: u32 = 0;
}

fn create_terminal_cell_data(cols: i32, rows: i32) -> Vec<CellDynamic> {
    let mut rng = SimpleRng::default();
    (0..cols * rows)
        .map(|_| CellDynamic::new(0, rng.gen(), rng.gen()))
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    const A: u32 = 1664525;
    const C: u32 = 1013904223;

    pub fn new(seed: u32) -> Self {
        SimpleRng { state: seed }
    }

    pub fn gen(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }
}

impl Default for SimpleRng {
    fn default() -> Self {
        let seed = web_time::SystemTime::now()
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32;

        SimpleRng::new(seed)
    }
}

mod attrib {
    pub const POS: u32 = 0;
    pub const UV: u32 = 1;

    pub const GRID_XY: u32 = 2;
    pub const DEPTH: u32 = 3;
    pub const FG: u32 = 4;
    pub const BG: u32 = 5;
}