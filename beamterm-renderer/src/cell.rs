#[derive(Debug, Clone, Copy, Default)]
pub struct CellQuery {
    mode: SelectionMode,
    start: Option<(u16, u16)>,
    end: Option<(u16, u16)>,
    trim_trailing_whitespace: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum  SelectionMode {
    #[default]
    Block,
    Linear
}

pub fn select(mode: SelectionMode) -> CellQuery {
    CellQuery { mode, ..CellQuery::default() }
}

impl CellQuery {
    pub fn start(mut self, start: (u16, u16)) -> Self {
        self.start = Some(start);
        self
    }

    pub fn end(mut self, end: (u16, u16)) -> Self {
        self.end = Some(end);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.start.is_none() || self.end.is_none()
    }

    pub fn mode(&self) -> SelectionMode {
        self.mode
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