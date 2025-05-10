use web_sys::wasm_bindgen::JsCast;
use crate::error::Error;
use crate::gl::VertexArray;
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

    // encodes the vertex data as x,y coordinates
    let vertices: [f32; 12] = [
        // x     y
         0.5,  0.5,  // ne
        -0.5, -0.5,  // sw
         0.5, -0.5,  // nw
         0.5,  0.5,  // ne
        -0.5,  0.5,  // se
        -0.5, -0.5,  // sw
    ];
    // position attribute is at location 0
    let vertex_array = VertexArray::builder()
        .gl(&gl)
        .vertices(&vertices)
        .attribute_location(0)
        .components_per_vertex(2)
        .build()?;

    gl.clear_color(0.0, 0.0, 0.0, 1.0); // Black background
    gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

    // set the viewport to match canvas size
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    vertex_array.bind(&gl);
    vertex_array.draw(&gl);
    
    // draw a point
    // gl.draw_arrays(web_sys::WebGl2RenderingContext::POINTS, 0, 1);

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