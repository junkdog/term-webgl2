use compact_str::{CompactString, CompactStringExt};

use crate::gl::TerminalGrid;

#[derive(Debug, Clone, Copy, Default)]
pub struct CellQuery {
    pub(super) mode: SelectionMode,
    pub(super) start: Option<(u16, u16)>,
    pub(super) end: Option<(u16, u16)>,
    pub(super) trim_trailing_whitespace: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SelectionMode {
    #[default]
    Block,
    Linear,
}

/// Zero-allocation iterator over terminal cell indices.
#[derive(Debug)]
pub enum CellIterator {
    Block(BlockCellIterator),
    Linear(LinearCellIterator),
}

/// Iterator for block (rectangular) cell selection.
#[derive(Debug)]
pub struct BlockCellIterator {
    cols: u16,
    start: (u16, u16),
    end: (u16, u16),
    current: (u16, u16),
    finished: bool,
}

/// Iterator for linear cell selection.
#[derive(Debug)]
pub struct LinearCellIterator {
    cols: u16,
    current_idx: usize,
    end_idx: usize,
    finished: bool,
}

pub fn select(mode: SelectionMode) -> CellQuery {
    CellQuery { mode, ..CellQuery::default() }
}

impl CellQuery {
    pub fn start(mut self, start: (u16, u16)) -> Self {
        self.start = Some(start);
        self.end = Some(start);
        self
    }

    pub fn end(mut self, end: (u16, u16)) -> Self {
        self.end = Some(end);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    pub fn range(&self) -> Option<((u16, u16), (u16, u16))> {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            Some((
                (start.0.min(end.0), start.1.min(end.1)),
                (start.0.max(end.0), start.1.max(end.1)),
            ))
        } else {
            None
        }
    }

    pub fn trim_trailing_whitespace(mut self, enabled: bool) -> Self {
        self.trim_trailing_whitespace = enabled;
        self
    }
}

impl Iterator for CellIterator {
    type Item = (usize, bool); // (cell_index, needs_newline_after)

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CellIterator::Block(iter) => iter.next(),
            CellIterator::Linear(iter) => iter.next(),
        }
    }
}

impl Iterator for BlockCellIterator {
    type Item = (usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.current.1 > self.end.1 {
            return None;
        }

        let idx = self.current.1 as usize * self.cols as usize + self.current.0 as usize;

        // Check if we need a newline after this cell
        let is_end_of_row = self.current.0 == self.end.0;
        let is_last_row = self.current.1 == self.end.1;
        let needs_newline = is_end_of_row && !is_last_row;

        // Advance to next position
        if self.current.0 < self.end.0 {
            self.current.0 += 1;
        } else {
            self.current.0 = self.start.0;
            self.current.1 += 1;
            if self.current.1 > self.end.1 {
                self.finished = true;
            }
        }

        Some((idx, needs_newline))
    }
}

impl Iterator for LinearCellIterator {
    type Item = (usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.current_idx > self.end_idx {
            return None;
        }

        let idx = self.current_idx;

        // Check if we need a newline before this cell (except for the first cell)
        let is_row_start = idx % self.cols as usize == 0;
        let is_first_cell = idx == self.current_idx;
        let needs_newline_before = is_row_start && !is_first_cell;

        // For linear mode, we actually want newlines BEFORE cells at row starts
        // But the original code put newlines AFTER the previous cell
        // Let's adjust the logic to match the original behavior

        self.current_idx += 1;
        if self.current_idx > self.end_idx {
            self.finished = true;
        }

        // Check if NEXT cell will need a newline before it
        let needs_newline_after = if self.current_idx <= self.end_idx {
            self.current_idx % self.cols as usize == 0
        } else {
            false
        };

        Some((idx, needs_newline_after))
    }
}

impl BlockCellIterator {
    fn new(cols: u16, start: (u16, u16), end: (u16, u16), max_cells: usize) -> Self {
        // Bounds checking and coordinate ordering
        let start = (
            start.0.min((max_cells / cols as usize).saturating_sub(1) as u16),
            start.1.min((max_cells / cols as usize).saturating_sub(1) as u16),
        );
        let end = (
            end.0.min(cols.saturating_sub(1)),
            end.1.min((max_cells / cols as usize).saturating_sub(1) as u16),
        );
        let (start, end) = if start > end { (end, start) } else { (start, end) };

        Self {
            cols,
            start,
            end,
            current: start,
            finished: start.1 > end.1,
        }
    }
}

impl LinearCellIterator {
    fn new(cols: u16, start: (u16, u16), end: (u16, u16), max_cells: usize) -> Self {
        let cols_usize = cols as usize;

        // Bounds checking and coordinate ordering
        let start = (
            start.0.min(cols.saturating_sub(1)),
            start.1.min((max_cells / cols_usize).saturating_sub(1) as u16),
        );
        let end = (
            end.0.min(cols.saturating_sub(1)),
            end.1.min((max_cells / cols_usize).saturating_sub(1) as u16),
        );
        let (start, end) = if start > end { (end, start) } else { (start, end) };

        let start_idx = start.1 as usize * cols_usize + start.0 as usize;
        let end_idx = end.1 as usize * cols_usize + end.0 as usize;
        let end_idx = end_idx.min(max_cells.saturating_sub(1));
        let start_idx = start_idx.min(end_idx);

        Self {
            cols,
            current_idx: start_idx,
            end_idx,
            finished: start_idx > end_idx,
        }
    }
}

impl TerminalGrid {
    /// Zero-allocation iterator over cell indices for a given selection range and mode.
    ///
    /// # Returns
    /// Iterator yielding (cell_index, needs_newline_after) tuples.
    pub fn cell_iter(
        &self,
        start: (u16, u16),
        end: (u16, u16),
        mode: SelectionMode,
    ) -> CellIterator {
        let cols = self.terminal_size().0;
        let max_cells = self.cell_count();

        match mode {
            SelectionMode::Block => {
                CellIterator::Block(BlockCellIterator::new(cols, start, end, max_cells))
            },
            SelectionMode::Linear => {
                CellIterator::Linear(LinearCellIterator::new(cols, start, end, max_cells))
            },
        }
    }

    pub fn get_text(&self, selection: CellQuery) -> CompactString {
        if let Some((start, end)) = selection.range() {
            let text = self.get_symbols(self.cell_iter(start, end, selection.mode));

            if selection.trim_trailing_whitespace {
                text.lines().map(str::trim_end).join_compact("\n")
            } else {
                text
            }
        } else {
            CompactString::const_new("")
        }
    }
}
