use beamterm_data::{FontAtlasData, Glyph};
use compact_str::CompactString;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::gl::{CellData, FontAtlas, Renderer, TerminalGrid};

/// JavaScript wrapper for the terminal renderer
#[wasm_bindgen]
#[derive(Debug)]
pub struct BeamtermRenderer {
    renderer: Renderer,
    terminal_grid: TerminalGrid,
}

/// JavaScript wrapper for cell data
#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct JsCellData {
    symbol: CompactString,
    style_bits: u16,
    fg_color: u32,
    bg_color: u32,
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CellStyle {
    bits: u16,
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
        self.bits |= Glyph::BOLD_FLAG; // Clear style bits, set bold
        self
    }

    /// Add italic style
    #[wasm_bindgen]
    pub fn italic(mut self) -> CellStyle {
        self.bits |= Glyph::ITALIC_FLAG; // Clear style bits, set italic
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
impl JsCellData {
    #[wasm_bindgen(constructor)]
    pub fn new(symbol: String, style: &CellStyle, fg_color: u32, bg_color: u32) -> JsCellData {
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

        Ok(BeamtermRenderer { renderer, terminal_grid })
    }

    /// Get the terminal dimensions in cells
    #[wasm_bindgen]
    pub fn terminal_size(&self) -> Vec<u16> {
        let (cols, rows) = self.terminal_grid.terminal_size();
        vec![cols, rows]
    }

    /// Get the cell size in pixels
    #[wasm_bindgen]
    pub fn cell_size(&self) -> Vec<i32> {
        let (width, height) = self.terminal_grid.cell_size();
        vec![width, height]
    }

    /// Update a single cell (in memory only - call synchronize() to upload to GPU)
    #[wasm_bindgen]
    pub fn update_cell(
        &mut self,
        row: u16,
        col: u16,
        cell_data: &JsCellData,
    ) -> Result<(), JsValue> {
        let (cols, rows) = self.terminal_grid.terminal_size();

        if row >= rows || col >= cols {
            return Err(JsValue::from_str("Cell coordinates out of bounds"));
        }

        let cell = CellData::new_with_style_bits(
            &cell_data.symbol,
            cell_data.style_bits,
            cell_data.fg_color,
            cell_data.bg_color,
        );

        self.terminal_grid.update_cell(row, col, cell);
        Ok(())
    }

    /// Write text to the terminal starting at the specified position (in memory only - call synchronize() to upload to GPU)
    #[wasm_bindgen]
    pub fn write_text(
        &mut self,
        row: u16,
        col: u16,
        text: &str,
        style: &CellStyle,
        fg_color: u32,
        bg_color: u32,
    ) -> Result<(), JsValue> {
        let (cols, rows) = self.terminal_grid.terminal_size();

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

            self.terminal_grid.update_cell(row, current_col, cell);
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
        let (cols, rows) = self.terminal_grid.terminal_size();

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
                self.terminal_grid.update_cell(r, c, cell);
            }
        }

        Ok(())
    }

    /// Synchronize all cell updates to GPU buffers (call after cell updates, before render)
    #[wasm_bindgen]
    pub fn flush(&mut self) -> Result<(), JsValue> {
        let gl = self.renderer.gl();
        self.terminal_grid
            .flush_cells(gl)
            .map_err(|e| JsValue::from_str(&format!("Failed to synchronize buffers: {}", e)))?;
        Ok(())
    }

    /// Render the terminal to the canvas
    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.renderer.begin_frame();
        self.renderer.render(&self.terminal_grid);
        self.renderer.end_frame();
    }

    /// Resize the terminal to fit new canvas dimensions
    #[wasm_bindgen]
    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), JsValue> {
        let gl = self.renderer.gl();
        self.terminal_grid
            .resize(gl, (width, height))
            .map_err(|e| JsValue::from_str(&format!("Failed to resize: {}", e)))?;
        Ok(())
    }

    /// Clear the terminal with the specified background color (in memory only - call synchronize() to upload to GPU)
    #[wasm_bindgen]
    pub fn clear(&mut self, bg_color: u32) -> Result<(), JsValue> {
        let (cols, rows) = self.terminal_grid.terminal_size();

        for row in 0..rows {
            for col in 0..cols {
                let cell = CellData::new_with_style_bits(" ", 0, 0xFFFFFF, bg_color);
                self.terminal_grid.update_cell(row, col, cell);
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
