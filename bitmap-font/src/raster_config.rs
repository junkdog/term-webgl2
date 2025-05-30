use font_atlas::Glyph;

#[derive(Debug)]
pub(super) struct RasterizationConfig {
    pub(super) texture_width: i32,
    pub(super) texture_height: i32,
    pub(super) texture_depth: i32, // slices
    pub(super) cell_width: i32,
    pub(super) cell_height: i32,
}

impl RasterizationConfig {
    const GLYPHS_PER_SLICE: i32 = 16; // 4x4 grid
    const GRID_SIZE: i32 = 4;

    pub(super) fn new(
        cell_width: i32,
        cell_height: i32,
        glyphs: &[Glyph],
    ) -> Self {
        let slice_width = Self::GRID_SIZE * cell_width;
        let slice_height = Self::GRID_SIZE * cell_height;

        let max_id = glyphs.iter().map(|g| g.id).max().unwrap_or(0) as i32;
        let depth = (max_id + Self::GLYPHS_PER_SLICE - 1) / Self::GLYPHS_PER_SLICE;

        Self {
            texture_width: slice_width,
            texture_height: slice_height,
            texture_depth: next_pow2(depth),
            cell_width,
            cell_height,
        }
    }

    pub(super) fn texture_size(&self) -> usize {
        (self.texture_width * self.texture_height * self.texture_depth) as usize
    }
}


// Rounds up to the next power of 2
fn next_pow2(n: i32) -> i32 {
    let mut v = n;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    v
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_pow2() {
        assert_eq!(next_pow2(1), 1);
        assert_eq!(next_pow2(2), 2);
        assert_eq!(next_pow2(3), 4);
        assert_eq!(next_pow2(4), 4);
        assert_eq!(next_pow2(5), 8);
        assert_eq!(next_pow2(15), 16);
        assert_eq!(next_pow2(16), 16);
        assert_eq!(next_pow2(17), 32);
        assert_eq!(next_pow2(1023), 1024);
    }
}