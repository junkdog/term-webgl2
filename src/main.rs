use web_sys::wasm_bindgen::JsCast;
use crate::error::Error;
use crate::shaders::{BASIC_FRAGMENT_SHADER, BASIC_VERTEX_SHADER};

mod gl;
mod error;
mod shaders;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run().unwrap()
}

fn run() -> Result<(), Box<Error>> {
    let document = web_sys::window()
        .ok_or(Error::UnableToRetrieveWindow)
        .and_then(|w| w.document().ok_or(Error::UnableToRetrieveDocument))?;

    let canvas = canvas_by_selector(&document, "canvas")?;
    let gl = get_webgl2_rendering_context(&canvas)?;
    
    let shader = gl::ShaderProgram::create(&gl, BASIC_VERTEX_SHADER, BASIC_FRAGMENT_SHADER)?;
    shader.use_program(&gl);

    gl.clear_color(0.0, 0.0, 0.0, 1.0); // Black background
    gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

    // Set the viewport to match canvas size
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    // Draw a point
    gl.draw_arrays(web_sys::WebGl2RenderingContext::POINTS, 0, 1);

    Ok(())
}

fn get_webgl2_rendering_context(
    canvas: &web_sys::HtmlCanvasElement
) -> Result<web_sys::WebGl2RenderingContext, Error> {
    let gl = canvas.get_context("webgl2")
        .map_err(|_| Error::FailedToRetrieveWebGl2RenderingContext)?
        .ok_or(Error::FailedToRetrieveWebGl2RenderingContext)?
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .map_err(|_| Error::FailedToRetrieveWebGl2RenderingContext)?;
    Ok(gl)
}

fn canvas_by_selector(
    document: &web_sys::Document,
    id: &str
) -> Result<web_sys::HtmlCanvasElement, Error> {
    let canvas = document.query_selector(id)
        .map_err(|_| Error::UnableToRetrieveCanvas)?
        .ok_or(Error::UnableToRetrieveCanvas)?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| Error::UnableToRetrieveCanvas)?;

    Ok(canvas)
}