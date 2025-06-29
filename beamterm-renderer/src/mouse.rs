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
    gl::{SelectionTracker, TerminalDimensions},
    select, Error, SelectionMode, TerminalGrid,
};

pub(super) type MouseEventCallback = Box<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static>;
type EventHandler = Rc<RefCell<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static>>;

/// Handles mouse input events for a terminal grid.
///
/// Converts browser mouse events into terminal grid coordinates and manages
/// event handlers for mouse interactions. Maintains terminal dimensions for
/// accurate coordinate mapping.
pub struct TerminalMouseHandler {
    canvas: web_sys::HtmlCanvasElement,
    on_mouse_down: Closure<dyn FnMut(web_sys::MouseEvent)>,
    on_mouse_up: Closure<dyn FnMut(web_sys::MouseEvent)>,
    on_mouse_move: Closure<dyn FnMut(web_sys::MouseEvent)>,
    terminal_dimensions: crate::gl::TerminalDimensions,
    pub(crate) default_input_handler: Option<DefaultSelectionHandler>,
}

/// Mouse event data with terminal cell coordinates.
///
/// Represents a mouse event translated from pixel coordinates to terminal
/// grid coordinates, including modifier key states.
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

impl TerminalMouseHandler {
    /// Creates a new mouse handler for the given canvas and terminal grid.
    ///
    /// Sets up mouse event listeners on the canvas and converts pixel coordinates
    /// to terminal cell coordinates before invoking the provided event handler.
    ///
    /// # Arguments
    /// * `canvas` - The HTML canvas element to attach mouse listeners to
    /// * `grid` - The terminal grid for coordinate calculations
    /// * `event_handler` - Callback invoked for each mouse event
    ///
    /// # Errors
    /// Returns an error if event listeners cannot be attached to the canvas.
    pub fn new<F>(
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
        event_handler: MouseEventCallback,
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
            canvas: canvas.clone(),
            on_mouse_down,
            on_mouse_up,
            on_mouse_move,
            terminal_dimensions,
            default_input_handler: None,
        })
    }

    /// Updates the terminal dimensions after a resize.
    ///
    /// Must be called when the terminal grid is resized to ensure accurate
    /// coordinate conversion from pixels to cells.
    pub fn update_dimensions(&self, cols: u16, rows: u16) {
        self.terminal_dimensions.set(cols, rows);
    }

    /// Removes all owned event listeners from the canvas.
    ///
    /// This should be called before dropping the handler to prevent memory leaks
    /// and conflicts with new handlers.
    pub fn cleanup(&self) {
        let _ = self.canvas.remove_event_listener_with_callback(
            "mousedown",
            self.on_mouse_down.as_ref().unchecked_ref(),
        );
        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseup",
            self.on_mouse_up.as_ref().unchecked_ref(),
        );
        let _ = self.canvas.remove_event_listener_with_callback(
            "mousemove",
            self.on_mouse_move.as_ref().unchecked_ref(),
        );
    }
}

/// Default handler for mouse-based text selection and clipboard operations.
///
/// Implements standard terminal selection behavior: click and drag to select text,
/// automatic clipboard copy on selection completion. Supports both block and
/// linear selection modes.
pub(crate) struct DefaultSelectionHandler {
    selection_state: Rc<RefCell<SelectionState>>,
    grid: Rc<RefCell<TerminalGrid>>,
    query_mode: SelectionMode,
    trim_trailing_whitespace: bool,
}

impl DefaultSelectionHandler {
    /// Creates a new selection handler.
    ///
    /// # Arguments
    /// * `grid` - The terminal grid to select from
    /// * `query_mode` - Selection mode (block or linear)
    /// * `trim_trailing_whitespace` - Whether to trim whitespace from selected lines
    pub(crate) fn new(
        grid: Rc<RefCell<TerminalGrid>>,
        query_mode: SelectionMode,
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
    pub fn create_event_handler(&self, active_selection: SelectionTracker) -> MouseEventCallback {
        let selection_state = self.selection_state.clone();
        let query_mode = self.query_mode;
        let trim_trailing = self.trim_trailing_whitespace;

        Box::new(move |event: TerminalMouseEvent, grid: &TerminalGrid| {
            let mut state = selection_state.borrow_mut();

            match event.event_type {
                // only handle left mouse button events
                MouseEventType::MouseDown if event.button == 0 => {
                    // mouse down always begins a new *potential* selection
                    if state.is_complete() {
                        // the existing (completed) selection is replaced with
                        // a new selection which will be canceled if the mouse
                        // up event is fired on the same cell.
                        state.maybe_selecting(event.col, event.row);
                    } else {
                        // begins a new selection from a blank state
                        state.begin_selection(event.col, event.row);
                    }

                    let query = select(query_mode)
                        .start((event.col, event.row))
                        .trim_trailing_whitespace(trim_trailing);

                    active_selection.set_query(query);
                },
                MouseEventType::MouseMove if state.is_selecting() => {
                    state.update_selection(event.col, event.row);
                    active_selection.update_selection_end((event.col, event.row));
                },
                MouseEventType::MouseUp if event.button == 0 => {
                    // at this point, we're either at:
                    // a) the user has finished making the selection
                    // b) the selection was canceled by a click inside a single cell
                    if let Some((_start, _end)) = state.complete_selection(event.col, event.row) {
                        active_selection.update_selection_end((event.col, event.row));
                        let selected_text = grid.get_text(active_selection.query());
                        copy_to_clipboard(selected_text);
                    } else {
                        state.clear();
                        active_selection.clear();
                    }
                },
                _ => {}, // ignore non-left button events
            }
        })
    }
}

/// Internal state machine for tracking mouse selection operations.
///
/// Manages the lifecycle of a selection from initial click through dragging
/// to final release. Handles edge cases like single-cell clicks that should
/// cancel rather than select.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectionState {
    /// No selection in progress.
    Idle,
    /// Active selection with start point and current cursor position.
    Selecting {
        start: (u16, u16),
        current: Option<(u16, u16)>,
    },
    /// Potential selection that will be canceled if mouse up occurs on same cell.
    MaybeSelecting { start: (u16, u16) },
    /// Completed selection with final coordinates.
    Complete { start: (u16, u16), end: (u16, u16) },
}

impl SelectionState {
    /// Creates a new idle selection state.
    fn new() -> Self {
        SelectionState::Idle
    }

    /// Begins a new selection at the specified coordinates.
    fn begin_selection(&mut self, col: u16, row: u16) {
        *self = SelectionState::Selecting { start: (col, row), current: None };
    }

    /// Updates the current selection endpoint during dragging.
    fn update_selection(&mut self, col: u16, row: u16) {
        use SelectionState::*;

        match self {
            Selecting { current, .. } => {
                *current = Some((col, row));
            },
            MaybeSelecting { start } => {
                if (col, row) != *start {
                    *self = Selecting { start: *start, current: Some((col, row)) };
                }
            },
            _ => {},
        }
    }

    /// Checks if a selection is currently in progress.
    fn is_selecting(&self) -> bool {
        use SelectionState::*;
        matches!(self, Selecting { .. } | MaybeSelecting { .. })
    }

    /// Completes the selection at the specified coordinates.
    ///
    /// Returns the selection coordinates if valid, None if canceled.
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

    /// Resets the selection state to idle.
    fn clear(&mut self) {
        *self = SelectionState::Idle;
    }

    /// Enters a tentative selection state that may be canceled.
    fn maybe_selecting(&mut self, col: u16, row: u16) {
        *self = SelectionState::MaybeSelecting { start: (col, row) };
    }

    /// Checks if a selection has been completed.
    fn is_complete(&self) -> bool {
        matches!(self, SelectionState::Complete { .. })
    }
}

/// Creates a closure that handles browser mouse events and converts them to terminal events.
///
/// Wraps the event handler with coordinate conversion and terminal event creation logic.
fn create_mouse_event_closure(
    event_type: MouseEventType,
    grid: Rc<RefCell<TerminalGrid>>,
    event_handler: EventHandler,
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
            event_handler.borrow_mut()(terminal_event, &grid_ref);
        }
    }) as Box<dyn FnMut(_)>)
}

/// Copies text to the system clipboard using the browser's async clipboard API.
///
/// Spawns an async task to handle the clipboard write operation. Logs success
/// or failure to the console.
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
                    console::error_1(&format!("Failed to copy to clipboard: {err:?}").into());
                },
            }
        }
    });
}

impl Drop for TerminalMouseHandler {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl Debug for TerminalMouseHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (cols, rows) = self.terminal_dimensions.get();
        write!(f, "TerminalMouseHandler {{ dimensions: {cols}x{rows} }}")
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
