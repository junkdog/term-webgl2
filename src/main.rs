use web_sys::wasm_bindgen::JsCast;
use crate::error::Error;
use crate::gl::{CellArray, IndexedVertexArray, VertexArray};
use crate::shaders::{BASIC_FRAGMENT_SHADER, BASIC_VERTEX_SHADER};

mod gl;
mod error;
mod shaders;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run().unwrap()
}

fn run() -> Result<(), Error> {
    let document = web_sys::window()
        .ok_or(Error::UnableToRetrieveWindow)
        .and_then(|w| w.document().ok_or(Error::UnableToRetrieveDocument))?;

    let canvas = canvas_by_selector(&document, "canvas")?;
    let gl = get_webgl2_rendering_context(&canvas)?;
    
    // let shader = gl::ShaderProgram::create(&gl, BASIC_VERTEX_SHADER, BASIC_FRAGMENT_SHADER)?;
    let shader = gl::ShaderProgram::create(&gl, CellArray::VERTEX_GLSL, CellArray::FRAGMENT_GLSL)?;
    shader.use_program(&gl);

    // encodes the vertex data as x,y coordinates
    let vertices: [f32; 12] = [
        // x     y
         0.5,  0.5,  // ne 0
        -0.5, -0.5,  // sw 1
         0.5, -0.5,  // se 2
         0.5,  0.5,  // ne 0
        -0.5, -0.5,  // sw 1
        -0.5,  0.5,  // nw 3
    ];
    let vertices: [f32; 16] = [
        // x     y     u     v
         0.5,  0.5,  1.0,  1.0,  // ne 0
        -0.5, -0.5,  0.0,  0.0,  // sw 1
         0.5, -0.5,  1.0,  0.0,  // se 2
        -0.5,  0.5,  0.0,  1.0,  // nw 3
    ];

    let indices = [
        0, 1, 2, // first triangle
        0, 3, 1, // second triangle
    ];

    // position attribute is at location 0
    let vertex_array = CellArray::builder()
        .gl(&gl)
        .vertices(&vertices)
        .indices(&indices)
        .program(&shader.program)
        .build()?;
    
    // set the viewport to match canvas size
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    gl.clear_color(0.2, 0.2, 0.2, 1.0); // Black background
    gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

    vertex_array.draw(&gl);

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