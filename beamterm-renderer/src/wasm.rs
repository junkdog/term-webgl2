use std::{cell::RefCell, rc::Rc};

use beamterm_data::{FontAtlasData, Glyph};
use compact_str::CompactString;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::gl::{CellData, FontAtlas, Renderer, TerminalGrid};

/// JavaScript wrapper for the terminal renderer
#[wasm_bindgen]
#[derive(Debug)]
pub struct BeamtermRenderer {
    renderer: Renderer,
    terminal_grid: Rc<RefCell<TerminalGrid>>,
}

/// JavaScript wrapper for cell data
#[wasm_bindgen]
#[derive(Debug, Default, serde::Deserialize)]
pub struct Cell {
    symbol: CompactString,
    style: u16,
    fg: u32,
    bg: u32,
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Span {
    text: String,
    style: CellStyle,
    fg: Option<u32>,
    bg: Option<u32>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default)]
pub struct CellStyle {
    bits: u16,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Batch {
    // no functions exported when named `JsBatch`
    terminal_grid: Rc<RefCell<TerminalGrid>>,
    gl: web_sys::WebGl2RenderingContext,
}

#[wasm_bindgen]
impl CellStyle {
    /// Create a new TextStyle with default (normal) style
    #[wasm_bindgen(constructor)]
    pub fn new() -> CellStyle {
        CellStyle { bits: 0x0000 }
    }

    /// Add bold style
    #[wasm_bindgen]
    pub fn bold(mut self) -> CellStyle {
        self.bits |= Glyph::BOLD_FLAG;
        self
    }

    /// Add italic style
    #[wasm_bindgen]
    pub fn italic(mut self) -> CellStyle {
        self.bits |= Glyph::ITALIC_FLAG;
        self
    }

    /// Add underline effect
    #[wasm_bindgen]
    pub fn underline(mut self) -> CellStyle {
        self.bits |= Glyph::UNDERLINE_FLAG;
        self
    }

    /// Add strikethrough effect
    #[wasm_bindgen]
    pub fn strikethrough(mut self) -> CellStyle {
        self.bits |= Glyph::STRIKETHROUGH_FLAG;
        self
    }

    /// Get the combined style bits
    #[wasm_bindgen(getter)]
    pub fn bits(&self) -> u16 {
        self.bits
    }
}

#[wasm_bindgen]
impl Batch {
    /// Updates a single cell at the given position.
    #[wasm_bindgen(js_name = "putCell")]
    pub fn put_cell(&self, row: u16, col: u16, cell_data: &Cell) {
        self.terminal_grid.borrow_mut().update_cell(row, col, cell_data.as_cell_data());
    }

    /// Updates a cell by its buffer index.
    #[wasm_bindgen(js_name = "putCellByIndex")]
    pub fn put_cell_by_index(&self, idx: usize, cell_data: &Cell) {
        self.terminal_grid
            .borrow_mut()
            .update_cell_by_index(idx, cell_data.as_cell_data());
    }

    /// Updates multiple cells from an array.
    /// Each element should be [row, col, cellData].
    #[wasm_bindgen(js_name = "putCells")]
    pub fn put_cells(&mut self, cells_json: JsValue) -> Result<(), JsValue> {
        let updates = from_value::<Vec<(u16, u16, Cell)>>(cells_json)
            .map_err(|e| JsValue::from_str(&e.to_string()));

        match updates {
            Ok(cells) => {
                let cell_data =
                    cells.iter().map(|(row, col, data)| (*row, *col, data.as_cell_data()));

                let mut terminal_grid = self.terminal_grid.borrow_mut();
                terminal_grid
                    .update_cells_by_position(&self.gl, cell_data)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            },
            e => e.map(|_| ()),
        }
    }

    /// Write text to the terminal
    #[wasm_bindgen(js_name = "writeText")]
    pub fn write_text(
        &self,
        row: u16,
        col: u16,
        text: &str,
        style: &CellStyle,
        fg: u32,
        bg: u32,
    ) -> Result<(), JsValue> {
        let mut terminal_grid = self.terminal_grid.borrow_mut();
        let (cols, rows) = terminal_grid.terminal_size();

        if row >= rows {
            return Err(JsValue::from_str("Row out of bounds"));
        }

        for (i, ch) in text.chars().enumerate() {
            let current_col = col + i as u16;
            if current_col >= cols {
                break;
            }

            let ch_str = ch.to_string();
            let cell = CellData::new_with_style_bits(&ch_str, style.bits, fg, bg);
            terminal_grid.update_cell(row, current_col, cell);
        }

        Ok(())
    }

    /// Fill a rectangular region
    #[wasm_bindgen(js_name = "fillRect")]
    pub fn fill_rect(
        &self,
        row: u16,
        col: u16,
        width: u16,
        height: u16,
        cell_data: &Cell,
    ) -> Result<(), JsValue> {
        let mut terminal_grid = self.terminal_grid.borrow_mut();
        let (cols, rows) = terminal_grid.terminal_size();

        if row + height > rows || col + width > cols {
            return Err(JsValue::from_str("Rectangle extends beyond terminal bounds"));
        }

        for r in row..row + height {
            for c in col..col + width {
                terminal_grid.update_cell(r, c, cell_data.as_cell_data());
            }
        }

        Ok(())
    }

    /// Clear the terminal with specified background color
    #[wasm_bindgen]
    pub fn clear(&self, bg: u32) -> Result<(), JsValue> {
        let mut terminal_grid = self.terminal_grid.borrow_mut();
        let (cols, rows) = terminal_grid.terminal_size();

        for row in 0..rows {
            for col in 0..cols {
                let cell = CellData::new_with_style_bits(" ", 0, 0xFFFFFF, bg);
                terminal_grid.update_cell(row, col, cell);
            }
        }

        Ok(())
    }

    /// Synchronize all pending updates to the GPU
    #[wasm_bindgen]
    pub fn flush(&self) -> Result<(), JsValue> {
        self.terminal_grid
            .borrow_mut()
            .flush_cells(&self.gl)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
impl Cell {
    #[wasm_bindgen(constructor)]
    pub fn new(symbol: String, style: &CellStyle, fg: u32, bg: u32) -> Cell {
        Cell {
            symbol: symbol.into(),
            style: style.bits,
            fg,
            bg,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> String {
        self.symbol.to_string()
    }

    #[wasm_bindgen(setter)]
    pub fn set_symbol(&mut self, symbol: String) {
        self.symbol = symbol.into();
    }

    #[wasm_bindgen(getter)]
    pub fn fg(&self) -> u32 {
        self.fg
    }

    #[wasm_bindgen(setter)]
    pub fn set_fg(&mut self, color: u32) {
        self.fg = color;
    }

    #[wasm_bindgen(getter)]
    pub fn bg(&self) -> u32 {
        self.bg
    }

    #[wasm_bindgen(setter)]
    pub fn set_bg(&mut self, color: u32) {
        self.bg = color;
    }

    #[wasm_bindgen(getter)]
    pub fn style(&self) -> u16 {
        self.style
    }

    #[wasm_bindgen(setter)]
    pub fn set_style(&mut self, style: u16) {
        self.style = style;
    }
}

impl Cell {
    pub fn as_cell_data(&self) -> CellData {
        CellData::new_with_style_bits(&self.symbol, self.style, self.fg, self.bg)
    }
}

#[wasm_bindgen]
impl BeamtermRenderer {
    /// Create a new terminal renderer
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<BeamtermRenderer, JsValue> {
        console_error_panic_hook::set_once();

        let renderer = Renderer::create(canvas_id)
            .map_err(|e| JsValue::from_str(&format!("Failed to create renderer: {}", e)))?;

        let gl = renderer.gl();
        let atlas_data = FontAtlasData::default();
        let atlas = FontAtlas::load(gl, atlas_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to load font atlas: {}", e)))?;

        let canvas_size = renderer.canvas_size();
        let terminal_grid = TerminalGrid::new(gl, atlas, canvas_size)
            .map_err(|e| JsValue::from_str(&format!("Failed to create terminal grid: {}", e)))?;

        console::log_1(&"BeamtermRenderer initialized successfully".into());
        let terminal_grid = Rc::new(RefCell::new(terminal_grid));
        Ok(BeamtermRenderer { renderer, terminal_grid })
    }

    /// Create a new render batch
    #[wasm_bindgen(js_name = "batch")]
    pub fn new_render_batch(&mut self) -> Batch {
        let gl = self.renderer.gl().clone();
        let terminal_grid = self.terminal_grid.clone();
        Batch { terminal_grid, gl }
    }

    /// Get the terminal dimensions in cells
    #[wasm_bindgen(js_name = "terminalSize")]
    pub fn terminal_size(&self) -> Size {
        let (cols, rows) = self.terminal_grid.borrow().terminal_size();
        Size { width: cols, height: rows }
    }

    /// Get the cell size in pixels
    #[wasm_bindgen(js_name = "cellSize")]
    pub fn cell_size(&self) -> Size {
        let (width, height) = self.terminal_grid.borrow().cell_size();
        Size {
            width: width as u16,
            height: height as u16,
        }
    }

    /// Render the terminal to the canvas
    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.renderer.begin_frame();
        let grid: &TerminalGrid = &self.terminal_grid.borrow();
        self.renderer.render(grid);
        self.renderer.end_frame();
    }

    /// Resize the terminal to fit new canvas dimensions
    #[wasm_bindgen]
    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), JsValue> {
        self.renderer.resize(width, height);

        console::log_1(&format!("Resizing terminal to {}x{}", width, height).into());

        let gl = self.renderer.gl();
        self.terminal_grid
            .borrow_mut()
            .resize(gl, (width, height))
            .map_err(|e| JsValue::from_str(&format!("Failed to resize: {}", e)))?;
        Ok(())
    }
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"beamterm WASM module loaded".into());
}
