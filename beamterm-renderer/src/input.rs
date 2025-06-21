use std::{cell::RefCell, fmt::Debug, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast};

use crate::{Error, TerminalGrid};

pub struct TerminalInputHandler {
    mouse_down: Closure<dyn FnMut(web_sys::MouseEvent)>,
    mouse_up: Closure<dyn FnMut(web_sys::MouseEvent)>,
    mouse_move: Closure<dyn FnMut(web_sys::MouseEvent)>,
    terminal_size: DynamicSize,
}

struct DynamicSize {
    dimensions: Rc<RefCell<(u16, u16)>>,
}

impl Debug for TerminalInputHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TerminalInputHandler {{ ... }}")
    }
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
        grid: &TerminalGrid,
        mut callback: F,
    ) -> Result<Self, Error>
    where
        F: FnMut(TerminalMouseEvent) + 'static,
    {
        // Get grid metrics once
        let (cell_width, cell_height) = grid.cell_size();
        let (cols, rows) = grid.terminal_size();
        let terminal_size = DynamicSize::new(cols, rows);

        // Wrap callback for sharing
        let callback = Rc::new(RefCell::new(callback));

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

        use TerminalEventType::*;
        let mouse_down = make_callback(MouseDown, callback.clone(), coord_converter.clone());
        let mouse_up = make_callback(MouseUp, callback.clone(), coord_converter.clone());
        let mouse_move = make_callback(MouseMove, callback.clone(), coord_converter);

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

#[derive(Debug, Clone, Copy)]
pub struct TerminalMouseEvent {
    pub event_type: TerminalEventType,
    pub col: u16,
    pub row: u16,
    pub button: i16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalEventType {
    MouseDown,
    MouseUp,
    MouseMove,
}

fn make_callback<F>(
    event_type: TerminalEventType,
    callback: Rc<RefCell<F>>,
    coord_converter: impl Fn(&web_sys::MouseEvent) -> Option<(u16, u16)> + 'static,
) -> Closure<dyn FnMut(web_sys::MouseEvent)>
where
    F: FnMut(TerminalMouseEvent) + 'static,
{
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
            callback.borrow_mut()(terminal_event);
        }
    }) as Box<dyn FnMut(_)>)
}
