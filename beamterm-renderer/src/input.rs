use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    rc::Rc,
};

use compact_str::CompactString;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

use crate::{
    cell::{select, CellQuery, SelectionMode as QueryMode},
    Error, TerminalGrid,
};

/// Handles mouse input events for a terminal grid.
pub struct TerminalMouseHandler {
    on_mouse_down: Closure<dyn FnMut(web_sys::MouseEvent)>,
    on_mouse_up: Closure<dyn FnMut(web_sys::MouseEvent)>,
    on_mouse_move: Closure<dyn FnMut(web_sys::MouseEvent)>,
    terminal_dimensions: TerminalDimensions,
    pub(super) default_input_handler: Option<DefaultSelectionHandler>,
}

/// Mouse event data with terminal cell coordinates.
#[derive(Debug, Clone, Copy)]
pub struct TerminalMouseEvent {
    /// Type of mouse event (down, up, or move).
    pub event_type: MouseEventType,
    /// Column in the terminal grid (0-based).
    pub col: u16,
    /// Row in the terminal grid (0-based).
    pub row: u16,
    /// Mouse button pressed (0 = left, 1 = middle, 2 = right).
    pub button: i16,
    /// Whether Ctrl key was pressed during the event.
    pub ctrl_key: bool,
    /// Whether Shift key was pressed during the event.
    pub shift_key: bool,
    /// Whether Alt key was pressed during the event.
    pub alt_key: bool,
}

/// Types of mouse events that can occur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEventType {
    /// Mouse button was pressed.
    MouseDown,
    /// Mouse button was released.
    MouseUp,
    /// Mouse moved while over the terminal.
    MouseMove,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveSelection {
    query: Rc<RefCell<Option<CellQuery>>>,
}

#[derive(Debug, Clone)]
enum SelectionState {
    Idle,
    Selecting {
        start: (u16, u16),
        current: Option<(u16, u16)>,
    },
    Complete {
        start: (u16, u16),
        end: (u16, u16),
    },
}

impl SelectionState {
    fn new() -> Self {
        SelectionState::Idle
    }

    fn begin_selection(&mut self, col: u16, row: u16) {
        *self = SelectionState::Selecting { start: (col, row), current: None };
    }

    fn update_selection(&mut self, col: u16, row: u16) {
        if let SelectionState::Selecting { start: _, current } = self {
            *current = Some((col, row));
        }
    }

    fn complete_selection(&mut self, col: u16, row: u16) -> Option<((u16, u16), (u16, u16))> {
        match self {
            SelectionState::Selecting { start, .. } => {
                let result = Some((*start, (col, row)));
                *self = SelectionState::Complete { start: *start, end: (col, row) };
                result
            },
            _ => None,
        }
    }

    fn clear(&mut self) {
        *self = SelectionState::Idle;
    }

    fn is_complete(&self) -> bool {
        matches!(self, SelectionState::Complete { .. })
    }
}

struct TerminalDimensions {
    size: Rc<RefCell<(u16, u16)>>,
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

impl TerminalMouseHandler {
    /// Creates a new mouse handler for the given canvas and terminal grid.
    ///
    /// # Arguments
    /// * `canvas` - The HTML canvas element to attach mouse listeners to
    /// * `grid` - The terminal grid for coordinate calculations
    /// * `event_handler` - Callback invoked for each mouse event
    pub(crate) fn new<F>(
        canvas: &web_sys::HtmlCanvasElement,
        grid: Rc<RefCell<TerminalGrid>>,
        event_handler: F,
    ) -> Result<Self, Error>
    where
        F: FnMut(TerminalMouseEvent, &TerminalGrid) + 'static,
    {
        Self::new_internal(canvas, grid, Box::new(event_handler))
    }

    fn new_internal(
        canvas: &web_sys::HtmlCanvasElement,
        grid: Rc<RefCell<TerminalGrid>>,
        event_handler: Box<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static>,
    ) -> Result<Self, Error> {
        // Wrap the handler in Rc<RefCell> for sharing between closures
        let shared_handler = Rc::new(RefCell::new(event_handler));

        // Get grid metrics for coordinate conversion
        let (cell_width, cell_height) = grid.borrow().cell_size();
        let (cols, rows) = grid.borrow().terminal_size();
        let terminal_dimensions = TerminalDimensions::new(cols, rows);

        // Create pixel-to-cell coordinate converter
        let dimensions_ref = terminal_dimensions.clone_ref();
        let pixel_to_cell = move |event: &web_sys::MouseEvent| -> Option<(u16, u16)> {
            let x = event.offset_x() as f32;
            let y = event.offset_y() as f32;

            let col = (x / cell_width as f32).floor() as u16;
            let row = (y / cell_height as f32).floor() as u16;

            let (max_cols, max_rows) = *dimensions_ref.borrow();
            if col < max_cols && row < max_rows {
                Some((col, row))
            } else {
                None
            }
        };

        // Create event handlers
        use MouseEventType::*;
        let on_mouse_down = create_mouse_event_closure(
            MouseDown,
            grid.clone(),
            shared_handler.clone(),
            pixel_to_cell.clone(),
        );
        let on_mouse_up = create_mouse_event_closure(
            MouseUp,
            grid.clone(),
            shared_handler.clone(),
            pixel_to_cell.clone(),
        );
        let on_mouse_move =
            create_mouse_event_closure(MouseMove, grid.clone(), shared_handler, pixel_to_cell);

        // Attach event listeners
        canvas
            .add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())
            .map_err(|_| Error::Callback("Failed to add mousedown listener".into()))?;
        canvas
            .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())
            .map_err(|_| Error::Callback("Failed to add mouseup listener".into()))?;
        canvas
            .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())
            .map_err(|_| Error::Callback("Failed to add mousemove listener".into()))?;

        Ok(Self {
            on_mouse_down,
            on_mouse_up,
            on_mouse_move,
            terminal_dimensions,
            default_input_handler: None,
        })
    }

    /// Updates the terminal dimensions after a resize.
    pub(crate) fn update_dimensions(&self, cols: u16, rows: u16) {
        self.terminal_dimensions.set(cols, rows);
    }
}

/// Active selection state that tracks the current selection query.
impl ActiveSelection {
    /// Creates a new active selection instance.
    ///
    /// # Arguments
    /// * `query` - The selection query to track
    pub(super) fn new() -> Self {
        Self { query: Rc::new(RefCell::new(None)) }
    }

    /// Clears the current selection.
    fn clear(&self) {
        *self.query.borrow_mut() = None;
    }

    /// Returns the current selection query.
    fn query(&self) -> CellQuery {
        self.query
            .borrow()
            .clone()
            .expect("query to be a value due to internal-only usage")
    }

    /// Sets a new selection query.
    fn set_query(&self, query: CellQuery) {
        *self.query.borrow_mut() = Some(query);
    }

    fn update_selection_end(&self, end: (u16, u16)) {
        if let Some(query) = self.query.borrow_mut().as_mut() {
            *query = query.end(end);
        }
    }
}

/// Default handler for mouse-based text selection and clipboard operations.
pub(super) struct DefaultSelectionHandler {
    selection_state: Rc<RefCell<SelectionState>>,
    grid: Rc<RefCell<TerminalGrid>>,
    query_mode: QueryMode,
    trim_trailing_whitespace: bool,
}

impl DefaultSelectionHandler {
    /// Creates a new selection handler.
    ///
    /// # Arguments
    /// * `grid` - The terminal grid to select from
    /// * `query_mode` - Selection mode (block or linear)
    /// * `trim_trailing_whitespace` - Whether to trim whitespace from selected lines
    pub(super) fn new(
        grid: Rc<RefCell<TerminalGrid>>,
        query_mode: QueryMode,
        trim_trailing_whitespace: bool,
    ) -> Self {
        Self {
            selection_state: Rc::new(RefCell::new(SelectionState::new())),
            grid,
            query_mode,
            trim_trailing_whitespace,
        }
    }

    /// Creates an event handler function for mouse events.
    ///
    /// Returns a boxed closure that handles mouse events, tracks selection state,
    /// and copies selected text to the clipboard on completion.
    pub(crate) fn create_event_handler(
        &self,
        active_selection: ActiveSelection,
    ) -> Box<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static> {
        let selection_state = self.selection_state.clone();
        let query_mode = self.query_mode;
        let trim_trailing_whitespace = self.trim_trailing_whitespace;

        Box::new(move |event: TerminalMouseEvent, grid: &TerminalGrid| {
            let mut state = selection_state.borrow_mut();

            match event.event_type {
                MouseEventType::MouseDown => {
                    if event.button == 0 {
                        if state.is_complete() {
                            state.clear();
                            active_selection.clear();
                        } else {
                            state.begin_selection(event.col, event.row);
                            let mut query = select(query_mode).start((event.col, event.row));
                            if trim_trailing_whitespace {
                                query = query.trim_trailing_whitespace();
                            }

                            active_selection.set_query(query);
                        }
                    }
                },
                MouseEventType::MouseMove => {
                    state.update_selection(event.col, event.row);
                    active_selection.update_selection_end((event.col, event.row));
                },
                MouseEventType::MouseUp => {
                    if event.button == 0 {
                        if let Some((_, end)) = state.complete_selection(event.col, event.row) {
                            active_selection.update_selection_end(end);

                            let selected_text = grid.get_text(active_selection.query());
                            copy_to_clipboard(selected_text);
                        }
                    }
                },
            }
        })
    }
}

fn create_mouse_event_closure(
    event_type: MouseEventType,
    grid: Rc<RefCell<TerminalGrid>>,
    event_handler: Rc<RefCell<Box<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static>>>,
    pixel_to_cell: impl Fn(&web_sys::MouseEvent) -> Option<(u16, u16)> + 'static,
) -> Closure<dyn FnMut(web_sys::MouseEvent)> {
    Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if let Some((col, row)) = pixel_to_cell(&event) {
            let terminal_event = TerminalMouseEvent {
                event_type,
                col,
                row,
                button: event.button(),
                ctrl_key: event.ctrl_key(),
                shift_key: event.shift_key(),
                alt_key: event.alt_key(),
            };
            let grid_ref = grid.borrow();
            event_handler.borrow_mut()(terminal_event, &*grid_ref);
        }
    }) as Box<dyn FnMut(_)>)
}

fn copy_to_clipboard(text: CompactString) {
    console::log_1(&format!("Copying {} characters to clipboard", text.len()).into());

    spawn_local(async move {
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            match wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text)).await {
                Ok(_) => {
                    console::log_1(
                        &format!("Successfully copied {} characters", text.chars().count()).into(),
                    );
                },
                Err(err) => {
                    console::error_1(&format!("Failed to copy to clipboard: {:?}", err).into());
                },
            }
        }
    });
}

impl Debug for TerminalMouseHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (cols, rows) = self.terminal_dimensions.get();
        write!(f, "TerminalMouseHandler {{ dimensions: {}x{} }}", cols, rows)
    }
}

impl Debug for DefaultSelectionHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (cols, rows) = self.grid.borrow().terminal_size();
        write!(
            f,
            "DefaultSelectionHandler {{ mode: {:?}, trim_whitespace: {}, grid: {}x{} }}",
            self.query_mode, self.trim_trailing_whitespace, cols, rows
        )
    }
}
