use web_sys::console;
use crate::error::Error;
use crate::gl::{CellArray, Renderer, Texture, TextureAtlas, GL};
use crate::mat4::Mat4;

mod gl;
mod error;
mod shaders;
mod mat4;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run().unwrap()
}

fn run() -> Result<(), Error> {
    
    let mut renderer = Renderer::create("canvas")?;
    let shader = renderer.create_shader_program(CellArray::VERTEX_GLSL, CellArray::FRAGMENT_GLSL)?;

    let projection = Mat4::orthographic_from_size(
        renderer.canvas_width() as f32,
        renderer.canvas_height() as f32
    );

    // create texture
    const PIXELS: &[u8] = include_bytes!("../../data/bitmap_font.png");
    let texture = Texture::from_image_data(renderer.gl(), GL::RGBA, PIXELS)?;
    let atlas = TextureAtlas::from_grid(texture)?;

    let region = atlas.get_region(8).unwrap();
    let (u1, v1, u2, v2) = region.uvs;
    console::log_1(&format!("{:?}", region).into());
    let vertices: [f32; 16] = [
        //  x      y      u     v
        500.0, 100.0,  u2, 1.0 - v1, //0.25,  1.0,  // top-right
        100.0, 500.0,  u1, 1.0 - v2, //0.0,   0.0,  // bottom-left
        500.0, 500.0,  u2, 1.0 - v2, //0.25,  0.0,  // bottom-right
        100.0, 100.0,  u1, 1.0 - v1, //0.0,   1.0,  // top-left
    ];

    let indices = [
        0, 1, 2, // first triangle
        0, 3, 1, // second triangle
    ];

    let vertex_array = CellArray::builder()
        .gl(renderer.gl())
        .vertices(&vertices)
        .indices(&indices)
        .shader(&shader)
        .atlas(atlas)
        .build()?;
    
    
    
    renderer.clear(0.2, 0.2, 0.2);
    renderer.state()
        .blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
    shader.set_uniform_mat4(renderer.gl(), "u_projection", &projection)?;
    renderer.draw(&shader, &vertex_array);

    Ok(())
}
