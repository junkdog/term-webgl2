use crate::raster_config::RasterizationConfig;
use font_atlas::FontAtlasData;

#[derive(Debug, Clone, Copy)]
pub(super) struct GlyphCoordinate {
    pub(super) slice: u16, // Z coordinate in 3D texture
    pub(super) grid_x: u8, // X position in 4x4 grid (0-3)
    pub(super) grid_y: u8, // Y position in 4x4 grid (0-3)
}

impl GlyphCoordinate {
    pub(super) fn from_glyph_id(id: u16) -> Self {
        // 16 glyphs per slice (4x4)
        let slice = id >> 4;
        let position_in_slice = id & 0xF;
        let grid_x = (position_in_slice % 4) as u8;
        let grid_y = (position_in_slice / 4) as u8;

        Self { slice, grid_x, grid_y }
    }

    pub(super) fn xy(&self, config: &RasterizationConfig) -> (i32, i32) {
        let x = self.grid_x as i32 * config.cell_width + FontAtlasData::PADDING;
        let y = self.grid_y as i32 * config.cell_height + FontAtlasData::PADDING;
        (x, y)
    }

    pub(super) fn cell_offset(&self, config: &RasterizationConfig) -> (i32, i32, i32) {
        (
            self.grid_x as i32 * config.cell_width,
            self.grid_y as i32 * config.cell_height,
            self.slice as i32,
        )
    }
}

