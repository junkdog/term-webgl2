use fontdue::{Font, FontSettings};
use web_sys::wasm_bindgen::JsCast;
use crate::error::Error;
use crate::gl::{CellArray, Renderer};

mod gl;
mod error;
mod shaders;
mod bitmap_font;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run().unwrap()
}

fn run() -> Result<(), Error> {
    
    let mut renderer = Renderer::create("canvas")?;
    let shader = renderer.create_shader_program(CellArray::VERTEX_GLSL, CellArray::FRAGMENT_GLSL)?;

    let vertices: [f32; 16] = [
        //  x      y      u    v
         0.75,  0.75,  0.25, 1.0,  // ne 0
        -0.75, -0.75,  0.0,  0.0,  // sw 1
         0.75, -0.75,  0.25, 0.0,  // se 2
        -0.75,  0.75,  0.0,  1.0,  // nw 3
    ];

    let indices = [
        0, 1, 2, // first triangle
        0, 3, 1, // second triangle
    ];

    let vertex_array = CellArray::builder()
        .gl(renderer.gl())
        .vertices(&vertices)
        .indices(&indices)
        .program(&shader.program)
        .build()?;
    
    renderer.clear(0.2, 0.2, 0.2);
    renderer.state()
        .blend_func(gl::GL::SRC_ALPHA, gl::GL::ONE_MINUS_SRC_ALPHA);
    renderer.draw(&shader, &vertex_array);

    Ok(())
}
