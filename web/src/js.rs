use js_sys::wasm_bindgen::JsCast;
use web_sys::{Document, HtmlCanvasElement};
use crate::error::Error;

pub(crate) fn document() -> Result<Document, Error> {
    web_sys::window()
        .ok_or(Error::UnableToRetrieveWindow)
        .and_then(|w| w.document().ok_or(Error::UnableToRetrieveDocument))
}

pub(crate) fn create_canvas(width: u32, height: u32) -> Result<HtmlCanvasElement, Error> {
    let document = document()?;
    document.create_element("canvas")
        .map_err(|_| Error::UnableToCreateElement("canvas"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| Error::UnableToCreateElement("canvas"))
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

pub(crate) fn get_2d_context(
    canvas: &HtmlCanvasElement,
) -> Result<web_sys::CanvasRenderingContext2d, Error> {
    canvas.get_context("2d")
        .map_err(|_| Error::FailedToRetrieveCanvasRenderingContext2d)?
        .ok_or(Error::FailedToRetrieveCanvasRenderingContext2d)?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| Error::FailedToRetrieveWebGl2RenderingContext)
}
