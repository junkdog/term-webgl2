use crate::error::Error;
use crate::gl::ubo::UniformBufferObject;
use crate::gl::{buffer_upload_array, Drawable, FontAtlas, RenderContext, ShaderProgram, GL};
use crate::mat4::Mat4;
use std::fmt::Debug;
use web_sys::{console, WebGl2RenderingContext};

/// A high-performance terminal grid renderer using instanced rendering.
///
/// `TerminalGrid` renders a grid of terminal cells using WebGL2 instanced drawing.
/// Each cell can display a character from a font atlas with customizable foreground
/// and background colors. The renderer uses a 2D texture array to efficiently
/// store glyph data and supports real-time updates of cell content.
#[derive(Debug)]
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

#[derive(Debug)]
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
            .ok_or(Error::uniform_location_failed("u_sampler"))?;

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

    /// Uploads uniform buffer data for screen and cell dimensions.
    ///
    /// This method updates the shader uniform buffer with the current screen
    /// size and cell dimensions. Must be called when the screen size changes
    /// or when initializing the grid.
    ///
    /// # Parameters
    /// * `gl` - WebGL2 rendering context
    /// * `screen_size` - Screen dimensions in pixels as (width, height)
    pub fn upload_ubo_data(
        &self,
        gl: &WebGl2RenderingContext,
        screen_size: (i32, i32),
    ) {
        // let cell_size = (cell_size.0 - 2, cell_size.1 - 2);
        
        let cell_size = self.cell_size();
        
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

    /// Returns the total number of cells in the terminal grid.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Updates the content of terminal cells with new data.
    ///
    /// This method efficiently updates the dynamic instance buffer with new
    /// cell data. The iterator must provide exactly the same number of cells
    /// as the grid contains, in row-major order.
    ///
    /// # Parameters
    /// * `gl` - WebGL2 rendering context
    /// * `cells` - Iterator providing `CellData` for each cell in the grid
    ///
    /// # Returns
    /// * `Ok(())` - Successfully updated cell data
    /// * `Err(Error)` - Failed to update buffer or other WebGL error
    pub fn update_cells<'a>(
        &mut self,
        gl: &WebGl2RenderingContext,
        cells: impl Iterator<Item = CellData<'a>>,
    ) -> Result<(), Error> {
        // update instance buffer with new cell data
        let atlas = &self.atlas;

        let fallback_glyph = atlas.get_glyph_layer(" ").unwrap_or(0);
        self.cells.iter_mut()
            .zip(cells)
            .for_each(|(cell, data)| {
                let layer = atlas.get_glyph_layer(data.symbol).unwrap_or(fallback_glyph);
                *cell = CellDynamic::new(layer, data.fg, data.bg);
            });

        self.buffers.upload_instance_data(gl, &self.cells);

        Ok(())
    }
}

fn create_vao(gl: &WebGl2RenderingContext) -> Result<web_sys::WebGlVertexArrayObject, Error> {
    gl.create_vertex_array()
        .ok_or(Error::vertex_array_creation_failed())
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
        .ok_or(Error::buffer_creation_failed("vbo-u8"))?;
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
        .ok_or(Error::buffer_creation_failed("vbo-f32"))?;

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
        .ok_or(Error::buffer_creation_failed("static-instance-buffer"))?;


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
        .ok_or(Error::buffer_creation_failed("dynamic-instance-buffer"))?;
    
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&instance_buf));
    buffer_upload_array(gl, GL::ARRAY_BUFFER, instance_data, GL::DYNAMIC_DRAW);
    
    let stride = size_of::<CellDynamic>() as i32;

    // setup instance attributes (while VAO is bound)
    enable_vertex_attrib_array(gl, attrib::PACKED_DEPTH_FG_BG, 2, GL::UNSIGNED_INT, 0, stride);

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

/// Data for a single terminal cell including character and colors.
///
/// `CellData` represents the visual content of one terminal cell, including
/// the character to display and its foreground and background colors.
/// Colors are specified as ARGB values packed into 32-bit integers.
///
/// # Color Format
/// Colors use the format 0xAARRGGBB where:
/// - AA: Alpha channel (transparency)
/// - RR: Red component
/// - GG: Green component  
/// - BB: Blue component
#[derive(Debug)]
pub struct CellData<'a> {
    pub symbol: &'a str,
    pub fg: u32,
    pub bg: u32,
}

impl<'a> CellData<'a> {

    /// Creates new cell data with the specified character and colors.
    ///
    /// # Parameters
    /// * `symbol` - Character to display (should be a single character)
    /// * `fg` - Foreground color as ARGB value (0xAARRGGBB)
    /// * `bg` - Background color as ARGB value (0xAARRGGBB)
    ///
    /// # Returns
    /// New `CellData` instance
    pub fn new(symbol: &'a str, fg: u32, bg: u32) -> Self {
        Self { symbol, fg, bg }
    }
}


/// Static instance data for terminal cell positioning.
///
/// `CellStatic` represents the unchanging positional data for each terminal cell
/// in the grid. This data is uploaded once during initialization and remains
/// constant throughout the lifetime of the terminal grid. Each instance
/// corresponds to one cell position in the terminal grid.
///
/// # Memory Layout
/// This struct uses `#[repr(C, align(4))]` to ensure:
/// - C-compatible memory layout for GPU buffer uploads
/// - 4-byte alignment for efficient GPU access
/// - Predictable field ordering (grid_xy at offset 0)
///
/// # GPU Usage
/// This data is used as per-instance vertex attributes in the vertex shader,
/// allowing the same cell geometry to be rendered at different grid positions
/// using instanced drawing.
///
/// # Buffer Upload
/// Uploaded to GPU using `GL::STATIC_DRAW` since positions don't change.
#[repr(C, align(4))]
struct CellStatic {
    /// Grid position as (x, y) coordinates in cell units.
    pub grid_xy: [u16; 2],
}

/// Dynamic instance data for terminal cell appearance.
///
/// `CellDynamic` contains the frequently-changing visual data for each terminal
/// cell, including the character glyph and colors. This data is updated whenever
/// cell content changes and is efficiently uploaded to the GPU using dynamic
/// buffer updates.
///
/// # Memory Layout
/// The 8-byte data array is packed as follows:
/// - Bytes 0-1: Glyph depth/layer index (u16, little-endian)
/// - Bytes 2-4: Foreground color RGB (3 bytes)
/// - Bytes 5-7: Background color RGB (3 bytes)
///
/// This compact layout minimizes GPU memory usage and allows efficient
/// instanced rendering of the entire terminal grid.
///
/// # Color Format
/// Colors are stored as RGB bytes (no alpha channel in the instance data).
/// The alpha channel is handled separately in the shader based on glyph
/// transparency from the texture atlas.
///
/// # GPU Usage
/// Uploaded as instance attributes and accessed in both vertex and fragment
/// shaders for character selection and color application.
///
/// # Buffer Upload
/// Uploaded to GPU using `GL::DYNAMIC_DRAW` for efficient updates.
#[derive(Debug)]
#[repr(C, align(4))]
struct CellDynamic {
    
    /// Packed cell data:
    /// 
    /// # Byte Layout
    /// - `data[0]`: Lower 8 bits of glyph depth/layer index
    /// - `data[1]`: Upper 8 bits of glyph depth/layer index  
    /// - `data[2]`: Foreground red component (0-255)
    /// - `data[3]`: Foreground green component (0-255)
    /// - `data[4]`: Foreground blue component (0-255)
    /// - `data[5]`: Background red component (0-255)
    /// - `data[6]`: Background green component (0-255)
    /// - `data[7]`: Background blue component (0-255)
    pub data: [u8; 8], // 2b layer, fg:rgb, bg:rgb
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

    pub(crate) fn new(layer: i32, fg: u32, bg: u32) -> Self {
        let layer = layer as u32;
        let mut data = [0; 8];

        data[0] = (layer & 0xFF) as u8;
        data[1] = ((layer >> 8) & 0xFF) as u8;

        data[2] = ((fg >> 24) & 0xFF) as u8; // R
        data[3] = ((fg >> 16) & 0xFF) as u8; // G
        data[4] = ((fg >> 8) & 0xFF) as u8;  // B

        data[5] = ((bg >> 24) & 0xFF) as u8; // R
        data[6] = ((bg >> 16) & 0xFF) as u8; // G
        data[7] = ((bg >> 8) & 0xFF) as u8;  // B

        Self { data }
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
    (0..cols * rows)
        .map(|_| CellDynamic::new(0, 0xffff_ffff, 0x0000_00ff))
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleRng {
    state: u32,
}

mod attrib {
    pub const POS: u32 = 0;
    pub const UV: u32 = 1;

    pub const GRID_XY: u32 = 2;
    pub const PACKED_DEPTH_FG_BG: u32 = 3;
}