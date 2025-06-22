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
    cell::{select, SelectionMode},
    Error, TerminalGrid,
};

pub struct TerminalInputHandler {
    mouse_down: Closure<dyn FnMut(web_sys::MouseEvent)>,
    mouse_up: Closure<dyn FnMut(web_sys::MouseEvent)>,
    mouse_move: Closure<dyn FnMut(web_sys::MouseEvent)>,
    terminal_size: DynamicSize,
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalMouseEvent {
    pub event_type: MouseEventType,
    pub col: u16,
    pub row: u16,
    pub button: i16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEventType {
    MouseDown,
    MouseUp,
    MouseMove,
}

#[derive(Debug)]
struct MouseHandler {
    left_button_state: MouseEventType,
    start: Option<(u16, u16)>,
    end: Option<(u16, u16)>,
}

struct DynamicSize {
    dimensions: Rc<RefCell<(u16, u16)>>,
}

impl DynamicSize {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            dimensions: Rc::new(RefCell::new((cols, rows))),
        }
    }

    pub fn set_size(&self, cols: u16, rows: u16) {
        *self.dimensions.borrow_mut() = (cols, rows);
    }

    pub fn get_size(&self) -> (u16, u16) {
        *self.dimensions.borrow()
    }
}

impl TerminalInputHandler {
    pub(crate) fn new<F>(
        canvas: &web_sys::HtmlCanvasElement,
        grid: Rc<RefCell<TerminalGrid>>,
        callback: F,
    ) -> Result<Self, Error>
    where
        F: FnMut(TerminalMouseEvent, &TerminalGrid) + 'static,
    {
        Self::new_managed_callback(canvas, grid, Box::new(callback))
    }

    pub(crate) fn new_managed_callback(
        canvas: &web_sys::HtmlCanvasElement,
        grid: Rc<RefCell<TerminalGrid>>,
        callback: Box<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static>,
    ) -> Result<Self, Error> {
        // Wrap the boxed callback in Rc<RefCell> for sharing
        let callback = Rc::new(RefCell::new(callback));

        // Get grid metrics once
        let (cell_width, cell_height) = grid.borrow().cell_size();
        let (cols, rows) = grid.borrow().terminal_size();
        let terminal_size = DynamicSize::new(cols, rows);

        // Create coordinate conversion closure
        let dimensions = terminal_size.dimensions.clone();
        let coord_converter = move |event: &web_sys::MouseEvent| -> Option<(u16, u16)> {
            let x = event.offset_x() as f32;
            let y = event.offset_y() as f32;

            let col = (x / cell_width as f32).floor() as u16;
            let row = (y / cell_height as f32).floor() as u16;
            let (cols, rows) = *dimensions.borrow();
            if col < cols && row < rows {
                Some((col, row))
            } else {
                None
            }
        };

        use MouseEventType::*;
        let mouse_down =
            canvas_callback(MouseDown, grid.clone(), callback.clone(), coord_converter.clone());
        let mouse_up =
            canvas_callback(MouseUp, grid.clone(), callback.clone(), coord_converter.clone());
        let mouse_move =
            canvas_callback(MouseMove, grid.clone(), callback.clone(), coord_converter);

        // Attach listeners
        canvas
            .add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref())
            .map_err(|_| Error::Callback("mousedown".into()))?;
        canvas
            .add_event_listener_with_callback("mouseup", mouse_up.as_ref().unchecked_ref())
            .map_err(|_| Error::Callback("mouseup".into()))?;
        canvas
            .add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
            .map_err(|_| Error::Callback("mousemove".into()))?;

        Ok(Self {
            mouse_down,
            mouse_up,
            mouse_move,
            terminal_size,
        })
    }

    pub(crate) fn set_terminal_size(&self, cols: u16, rows: u16) {
        self.terminal_size.set_size(cols, rows);
    }
}

pub(super) struct DefaultSelectionHandler {
    mouse_handler: Rc<RefCell<MouseHandler>>,
    grid: Rc<RefCell<TerminalGrid>>,
    selection_mode: SelectionMode,
    trim_trailing_whitespace: bool,
}

impl DefaultSelectionHandler {
    pub fn new(
        grid: Rc<RefCell<TerminalGrid>>,
        selection_mode: SelectionMode,
        trim_trailing_whitespace: bool,
    ) -> Self {
        let mouse_handler = MouseHandler::new();
        let mouse_handler = Rc::new(RefCell::new(mouse_handler));
        Self {
            mouse_handler,
            selection_mode,
            grid,
            trim_trailing_whitespace,
        }
    }

    pub(crate) fn callback(&self) -> Box<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static> {
        let mouse_handler = self.mouse_handler.clone();

        Box::new(move |event: TerminalMouseEvent, grid: &TerminalGrid| {
            let mut handler = mouse_handler.borrow_mut();
            match event.event_type {
                MouseEventType::MouseDown => {
                    if handler.end.is_none() {
                        handler.left_button_state = MouseEventType::MouseDown;
                        handler.set_start(event.col, event.row);

                        console::log_1(
                            &format!(
                                "Mouse event: {:?} at ({}, {})",
                                handler, event.col, event.row
                            )
                            .into(),
                        );
                    } else {
                        handler.reset();
                    }
                },
                MouseEventType::MouseUp => {
                    handler.left_button_state = MouseEventType::MouseUp;
                    if handler.start.is_some() {
                        handler.set_end(event.col, event.row);

                        console::log_1(
                            &format!(
                                "Mouse event: {:?} at ({}, {})",
                                handler, event.col, event.row
                            )
                            .into(),
                        );

                        if let (Some(start), Some(end)) = (handler.start, handler.end) {
                            // copy selection to clipboard
                            let query = select(SelectionMode::Block)
                                .start(start)
                                .end(end)
                                .trim_trailing_whitespace();

                            copy_to_clipboard(grid.get_text(query));
                        }
                    }
                },
                MouseEventType::MouseMove => {
                    if handler.start.is_some()
                        && handler.left_button_state == MouseEventType::MouseDown
                    {
                        handler.set_end(event.col, event.row);
                    }
                },
            }
        })
    }
}

fn copy_to_clipboard(text: CompactString) {
    web_sys::console::log_1(&"Copying to clipboard".to_string().into());
    spawn_local(async move {
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            match wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text)).await {
                Ok(_) => {
                    web_sys::console::log_1(
                        &format!("Copied to clipboard: {} symbols", text.chars().count()).into(),
                    );
                },
                Err(err) => {
                    web_sys::console::error_1(
                        &format!("Failed to copy to clipboard: {:?}", err).into(),
                    );
                },
            }
        }
    });
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            left_button_state: MouseEventType::MouseUp,
            start: None,
            end: None,
        }
    }

    pub fn reset(&mut self) {
        self.left_button_state = MouseEventType::MouseUp;
        self.start = None;
        self.end = None;
    }

    pub fn set_start(&mut self, col: u16, row: u16) {
        self.start = Some((col, row));
    }

    pub fn set_end(&mut self, col: u16, row: u16) {
        self.end = Some((col, row));
    }
}

impl Debug for TerminalInputHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (cols, rows) = self.terminal_size.get_size();
        write!(f, "TerminalInputHandler {{ {cols}x{rows} }}")
    }
}

impl Debug for DefaultSelectionHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (cols, rows) = self.grid.borrow().terminal_size();
        write!(
            f,
            "DefaultSelectionHandler {{ mode: {:?}, trim_trailing_whitespace: {}, grid: {}x{} }}",
            self.selection_mode, self.trim_trailing_whitespace, cols, rows
        )
    }
}

fn canvas_callback(
    event_type: MouseEventType,
    grid: Rc<RefCell<TerminalGrid>>,
    callback: Rc<RefCell<Box<dyn FnMut(TerminalMouseEvent, &TerminalGrid) + 'static>>>,
    coord_converter: impl Fn(&web_sys::MouseEvent) -> Option<(u16, u16)> + 'static,
) -> Closure<dyn FnMut(web_sys::MouseEvent)> {
    Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if let Some((col, row)) = coord_converter(&event) {
            let terminal_event = TerminalMouseEvent {
                event_type,
                col,
                row,
                button: event.button(),
                ctrl_key: event.ctrl_key(),
                shift_key: event.shift_key(),
                alt_key: event.alt_key(),
            };
            callback.borrow_mut()(terminal_event, &*grid.borrow());
        }
    }) as Box<dyn FnMut(_)>)
}
