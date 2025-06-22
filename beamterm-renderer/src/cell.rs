use compact_str::{CompactString, CompactStringExt};

use crate::TerminalGrid;

#[derive(Debug, Clone, Copy, Default)]
pub struct CellQuery {
    mode: SelectionMode,
    start: Option<(u16, u16)>,
    end: Option<(u16, u16)>,
    trim_trailing_whitespace: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SelectionMode {
    #[default]
    Block,
    Linear,
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
        self.order_start_end();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    pub fn range(&self) -> Option<((u16, u16), (u16, u16))> {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            Some((start, end))
        } else {
            None
        }
    }

    fn order_start_end(&mut self) {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            if start > end {
                self.start = Some(end);
                self.end = Some(start);
            }
        }
    }
}

impl TerminalGrid {
    pub fn get_text(&self, selection: CellQuery) -> CompactString {
        if let Some((start, end)) = selection.range() {
            let text = match selection.mode {
                SelectionMode::Block => self.get_symbols(start, end),
                SelectionMode::Linear => self.get_symbols_linear(start, end),
            };

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
