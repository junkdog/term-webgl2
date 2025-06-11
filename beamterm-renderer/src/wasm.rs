use std::cell::RefCell;
use std::rc::Rc;
use compact_str::CompactString;
use serde_wasm_bindgen::from_value;
use beamterm_data::{FontAtlasData, Glyph};
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::gl::{CellData, FontAtlas, Renderer, TerminalGrid};

/// JavaScript wrapper for the terminal renderer
#[wasm_bindgen(js_name = "BeamtermRenderer")]
#[derive(Debug)]
pub struct JsBeamtermRenderer {
    renderer: Renderer,
    terminal_grid: Rc<RefCell<TerminalGrid>>,
}

/// JavaScript wrapper for cell data
#[wasm_bindgen(js_name = "CellData")]
#[derive(Debug, Default, serde::Deserialize)]
pub struct JsCellData {
    symbol: CompactString,
    style_bits: u16,
    fg_color: u32,
    bg_color: u32,
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct JsSpan {
    text: String,
    style: JsCellStyle,
    fg_color: Option<u32>,
    bg_color: Option<u32>,
}

#[wasm_bindgen(js_name = "CellStyle")]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsCellStyle {
    bits: u16,
}

#[wasm_bindgen(js_name = "Size")]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsSize {
    pub width: u16,
    pub height: u16,
}

#[wasm_bindgen]
impl JsCellStyle {
    /// Create a new TextStyle with default (normal) style
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsCellStyle {
        JsCellStyle { bits: 0x0000 }
    }

    /// Add bold style
    #[wasm_bindgen]
    pub fn bold(mut self) -> JsCellStyle {
        self.bits |= Glyph::BOLD_FLAG; // Clear style bits, set bold
        self
    }

    /// Add italic style
    #[wasm_bindgen]
    pub fn italic(mut self) -> JsCellStyle {
        self.bits |= Glyph::ITALIC_FLAG; // Clear style bits, set italic
        self
    }

    /// Add underline effect
    #[wasm_bindgen]
    pub fn underline(mut self) -> JsCellStyle {
        self.bits |= Glyph::UNDERLINE_FLAG;
        self
    }

    /// Add strikethrough effect
    #[wasm_bindgen]
    pub fn strikethrough(mut self) -> JsCellStyle {
        self.bits |= Glyph::STRIKETHROUGH_FLAG;
        self
    }

    /// Get the combined style bits
    #[wasm_bindgen(getter)]
    pub fn bits(&self) -> u16 {
        self.bits
    }
}

#[wasm_bindgen(js_name = "Batch")]
#[derive(Debug)]
#[wasm_bindgen]
pub struct JsBatch {
    terminal_grid: Rc<RefCell<TerminalGrid>>,
    gl: web_sys::WebGl2RenderingContext,
}

#[wasm_bindgen]
impl JsBatch {

    /// Updates a single cell at the given position.
    #[wasm_bindgen(js_name = putCell)]
    pub fn put_cell(&self, row: u16, col: u16, cell_data: &JsCellData) {
        self.terminal_grid.borrow_mut()
            .update_cell(row, col, cell_data.as_cell_data());
    }

    /// Updates a cell by its buffer index.
    #[wasm_bindgen(js_name = putCellByIndex)]
    pub fn put_cell_by_index(&self, idx: usize, cell_data: &JsCellData) {
        self.terminal_grid.borrow_mut()
            .update_cell_by_index(idx, cell_data.as_cell_data());
    }

    /// Updates multiple cells from an array.
    /// Each element should be [row, col, cellData].
    #[wasm_bindgen(js_name = putCells)]
    pub fn put_cells(&mut self, cells_json: JsValue) -> Result<(), JsValue> {
        let updates = from_value::<Vec<(u16, u16, JsCellData)>>(cells_json)
            .map_err(|e| JsValue::from_str(&e.to_string()));

        match updates {
            Ok(cells) => {
                let cell_data = cells.iter()
                    .map(|(row, col, data)| (*row, *col, data.as_cell_data()));

                let mut terminal_grid = self.terminal_grid.borrow_mut();
                terminal_grid
                    .update_cells_by_position(&self.gl, cell_data)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            e => e.map(|_| ()),
        }
    }

    /// Flushes all pending updates to the GPU.
    pub fn flush(&self) {
        self.terminal_grid.borrow_mut().flush_cells(&self.gl)
            .unwrap_or_default();
    }
}



#[wasm_bindgen]
impl JsCellData {
    #[wasm_bindgen(constructor)]
    pub fn new(symbol: String, style: &JsCellStyle, fg_color: u32, bg_color: u32) -> JsCellData {
        JsCellData {
            symbol: symbol.into(),
            style_bits: style.bits,
            fg_color,
            bg_color,
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
    pub fn fg_color(&self) -> u32 {
        self.fg_color
    }

    #[wasm_bindgen(setter)]
    pub fn set_fg_color(&mut self, color: u32) {
        self.fg_color = color;
    }

    #[wasm_bindgen(getter)]
    pub fn bg_color(&self) -> u32 {
        self.bg_color
    }

    #[wasm_bindgen(setter)]
    pub fn set_bg_color(&mut self, color: u32) {
        self.bg_color = color;
    }

}

impl JsCellData {
    pub fn as_cell_data(&self) -> CellData {
        CellData::new_with_style_bits(
            &self.symbol,
            self.style_bits,
            self.fg_color,
            self.bg_color,
        )
    }
}

#[wasm_bindgen]
impl JsBeamtermRenderer {
    /// Create a new terminal renderer
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<JsBeamtermRenderer, JsValue> {
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
        Ok(JsBeamtermRenderer { renderer, terminal_grid })
    }

    #[wasm_bindgen]
    pub fn new_render_batch(
        &mut self,
    ) -> JsBatch {
        let gl = self.renderer.gl().clone();
        let terminal_grid = self.terminal_grid.clone();
        JsBatch { terminal_grid, gl }
    }

    /// Get the terminal dimensions in cells
    #[wasm_bindgen]
    pub fn terminal_size(&self) -> JsSize {
        let (cols, rows) = self.terminal_grid.borrow().terminal_size();
        JsSize { width: cols, height: rows }
    }

    /// Get the cell size in pixels
    #[wasm_bindgen]
    pub fn cell_size(&self) -> JsSize {
        let (width, height) = self.terminal_grid.borrow().cell_size();
        JsSize {
            width: width as u16,
            height: height as u16,
        }
    }

    /// Update a single cell (in memory only - call synchronize() to upload to GPU)
    #[wasm_bindgen]
    pub fn update_cell(
        &mut self,
        row: u16,
        col: u16,
        cell_data: &JsCellData,
    ) -> Result<(), JsValue> {
        let (cols, rows) = self.terminal_grid.borrow().terminal_size();

        if row >= rows || col >= cols {
            return Err(JsValue::from_str("Cell coordinates out of bounds"));
        }

        let cell = CellData::new_with_style_bits(
            &cell_data.symbol,
            cell_data.style_bits,
            cell_data.fg_color,
            cell_data.bg_color,
        );

        self.terminal_grid.borrow_mut().update_cell(row, col, cell);
        Ok(())
    }

    /// Write text to the terminal starting at the specified position (in memory only - call synchronize() to upload to GPU)
    #[wasm_bindgen]
    pub fn write_text(
        &mut self,
        row: u16,
        col: u16,
        text: &str,
        style: &JsCellStyle,
        fg_color: u32,
        bg_color: u32,
    ) -> Result<(), JsValue> {
        let (cols, rows) = self.terminal_grid.borrow_mut().terminal_size();

        if row >= rows {
            return Err(JsValue::from_str("Row out of bounds"));
        }

        for (i, ch) in text.chars().enumerate() {
            let current_col = col + i as u16;
            if current_col >= cols {
                break;
            }

            let ch_str = ch.to_string();
            let cell = CellData::new_with_style_bits(&ch_str, style.bits, fg_color, bg_color);

            self.terminal_grid.borrow_mut().update_cell(row, current_col, cell);
        }

        Ok(())
    }

    /// Fill a rectangular region with the same cell data
    #[wasm_bindgen]
    pub fn fill_rect(
        &mut self,
        row: u16,
        col: u16,
        width: u16,
        height: u16,
        cell_data: &JsCellData,
    ) -> Result<(), JsValue> {
        let (cols, rows) = self.terminal_grid.borrow_mut().terminal_size();

        if row + height > rows || col + width > cols {
            return Err(JsValue::from_str("Rectangle extends beyond terminal bounds"));
        }

        for r in row..row + height {
            for c in col..col + width {
                let cell = CellData::new_with_style_bits(
                    &cell_data.symbol,
                    cell_data.style_bits,
                    cell_data.fg_color,
                    cell_data.bg_color,
                );
                self.terminal_grid.borrow_mut().update_cell(r, c, cell);
            }
        }

        Ok(())
    }

    /// Synchronize all cell updates to GPU buffers (call after cell updates, before render)
    #[wasm_bindgen]
    pub fn flush(&mut self) -> Result<(), JsValue> {
        let gl = self.renderer.gl();
        self.terminal_grid
            .borrow_mut()
            .flush_cells(gl)
            .map_err(|e| JsValue::from_str(&format!("Failed to synchronize buffers: {}", e)))?;
        Ok(())
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

        let gl = self.renderer.gl();
        self.terminal_grid
            .borrow_mut()
            .resize(gl, (width, height))
            .map_err(|e| JsValue::from_str(&format!("Failed to resize: {}", e)))?;
        Ok(())
    }

    /// Clear the terminal with the specified background color (in memory only - call synchronize() to upload to GPU)
    #[wasm_bindgen]
    pub fn clear(&mut self, bg_color: u32) -> Result<(), JsValue> {
        let (cols, rows) = self.terminal_grid.borrow().terminal_size();

        for row in 0..rows {
            for col in 0..cols {
                let cell = CellData::new_with_style_bits(" ", 0, 0xFFFFFF, bg_color);
                self.terminal_grid.borrow_mut().update_cell(row, col, cell);
            }
        }

        Ok(())
    }
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"beamterm WASM module loaded".into());
}
