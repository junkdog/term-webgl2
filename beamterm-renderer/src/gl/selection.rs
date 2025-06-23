use std::{
    cell::{RefCell, RefMut},
    fmt::{Debug, Formatter},
    rc::Rc,
};

use compact_str::CompactString;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

use crate::{
    gl::{
        cell_query::{select, CellQuery, SelectionMode},
        TerminalGrid,
    },
    Error,
};

#[derive(Debug, Clone)]
pub(crate) struct SelectionTracker {
    query: Rc<RefCell<Option<CellQuery>>>,
}

pub(crate) struct TerminalDimensions {
    size: Rc<RefCell<(u16, u16)>>,
}

/// Active selection state that tracks the current selection query.
impl SelectionTracker {
    /// Creates a new active selection instance.
    ///
    /// # Arguments
    /// * `query` - The selection query to track
    pub(super) fn new() -> Self {
        Self { query: Rc::new(RefCell::new(None)) }
    }

    /// Clears the current selection.
    pub(crate) fn clear(&self) {
        *self.query.borrow_mut() = None;
    }

    /// Returns the current selection query.
    pub(crate) fn query(&self) -> CellQuery {
        self.get_query().expect("query to be a value due to internal-only usage")
    }

    pub(super) fn mode(&self) -> SelectionMode {
        self.query.borrow().as_ref().map_or(SelectionMode::default(), |q| q.mode)
    }

    /// Returns the current selection query or `None` if no selection is active.
    pub(super) fn get_query(&self) -> Option<CellQuery> {
        self.query.borrow().clone()
    }

    /// Sets a new selection query.
    pub(crate) fn set_query(&self, query: CellQuery) {
        *self.query.borrow_mut() = Some(query);
    }

    pub(crate) fn update_selection_end(&self, end: (u16, u16)) {
        if let Some(query) = self.query.borrow_mut().as_mut() {
            *query = query.end(end);
        }
    }
}

impl TerminalDimensions {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            size: Rc::new(RefCell::new((cols, rows))),
        }
    }

    pub fn set(&self, cols: u16, rows: u16) {
        *self.size.borrow_mut() = (cols, rows);
    }

    pub fn get(&self) -> (u16, u16) {
        *self.size.borrow()
    }

    pub fn clone_ref(&self) -> Rc<RefCell<(u16, u16)>> {
        self.size.clone()
    }
}
