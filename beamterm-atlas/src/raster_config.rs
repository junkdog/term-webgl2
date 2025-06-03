use beamterm_data::Glyph;

#[derive(Debug)]
pub(super) struct RasterizationConfig {
    pub(super) texture_width: i32,
    pub(super) texture_height: i32,
    pub(super) layers: i32, 
    pub(super) cell_width: i32,
    pub(super) cell_height: i32,
}

impl RasterizationConfig {
    const GLYPHS_PER_SLICE: i32 = 16; // 16x1 grid
    const GRID_WIDTH: i32 = 16;
    const GRID_HEIGHT: i32 = 1;

    pub(super) fn new(
        cell_width: i32,
        cell_height: i32,
        glyphs: &[Glyph],
    ) -> Self {
        let slice_width = Self::GRID_WIDTH * cell_width;
        let slice_height = Self::GRID_HEIGHT * cell_height;

        let max_id = glyphs.iter().map(|g| g.id).max().unwrap_or(0) as i32;
        let layers = (max_id + Self::GLYPHS_PER_SLICE - 1) / Self::GLYPHS_PER_SLICE;

        Self {
            texture_width: slice_width,
            texture_height: slice_height,
            layers,
            cell_width,
            cell_height,
        }
    }

    pub(super) fn texture_size(&self) -> usize {
        (self.texture_width * self.texture_height * self.layers) as usize
    }
}
