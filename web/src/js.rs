use js_sys::wasm_bindgen::JsCast;
use web_sys::{Document, HtmlCanvasElement};
use crate::error::Error;

pub(crate) fn document() -> Result<Document, Error> {
    web_sys::window()
        .ok_or(Error::UnableToRetrieveWindow)
        .and_then(|w| w.document().ok_or(Error::UnableToRetrieveDocument))
}

pub(crate) fn get_canvas_by_id(
    canvas_id: &str,
) -> Result<HtmlCanvasElement, Error> {
    let document = document()?;
    document.query_selector(canvas_id)
        .map_err(|_| Error::UnableToRetrieveCanvas)?
        .ok_or(Error::UnableToRetrieveCanvas)?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| Error::UnableToRetrieveCanvas)
}

pub(crate) fn get_webgl2_context(
    canvas: &HtmlCanvasElement,
) -> Result<web_sys::WebGl2RenderingContext, Error> {
    canvas.get_context("webgl2")
        .map_err(|_| Error::FailedToRetrieveWebGl2RenderingContext)?
        .ok_or(Error::FailedToRetrieveWebGl2RenderingContext)?
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .map_err(|_| Error::FailedToRetrieveWebGl2RenderingContext)
}

