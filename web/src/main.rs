use image::{metadata, DynamicImage, GenericImageView};
use js_sys::wasm_bindgen::JsCast;
use serde::Deserialize;
use web_sys::console;
use crate::bitmap_font::BitmapFontMetadata;
use crate::error::Error;
use crate::gl::{CellArray, InstanceData, Renderer, Texture, TextureAtlas, GL};
use crate::mat4::Mat4;

mod gl;
mod error;
mod shaders;
mod mat4;
mod bitmap_font;
mod js;

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
    const METADATA_JSON: &'static str = include_str!("../../data/bitmap_font.json");
    let metadata: BitmapFontMetadata = serde_json::from_str(METADATA_JSON).unwrap();
    let texture = Texture::from_image_data(renderer.gl(), GL::RGBA, PIXELS, &metadata)?;
    let atlas = TextureAtlas::from_bitmap_font(renderer.gl(), texture, &metadata)?;

    let region = atlas.get_region("B").unwrap();
    console::log_1(&format!("{:?}", region).into());
    // model data
    let (w, h) = (metadata.cell_width as f32, metadata.cell_height as f32);
    let model_data: [f32; 16] = [
        //  x      y     u     v
            w,   0.0,  1.0,  0.0,  // top-right
          0.0,     h,  0.0,  1.0,  // bottom-left
            w,     h,  1.0,  1.0,  // bottom-right
          0.0,   0.0,  0.0,  0.0,  // top-left
    ];
    
    let transform_data: [InstanceData; 6] = [
        // xy         // depth      // fg              bg
        InstanceData::new((0, 0), 0, 0xffffffff, 0x000000FF), 
        InstanceData::new((1, 0), 1, 0xffffffff, 0x000000FF),
        InstanceData::new((2, 1), 2, 0xffffffff, 0x000000FF), 
        InstanceData::new((3, 1), 3, 0xffffffff, 0x000000FF), 
        InstanceData::new((4, 1), 4, 0xffffffff, 0x000000FF), 
        InstanceData::new((5, 1), 5, 0xffffffff, 0x000000FF),
    ];

    let indices = [
        0, 1, 2, // first triangle
        0, 3, 1, // second triangle
    ];

    let vertex_array = CellArray::builder()
        .gl(renderer.gl())
        .model_data(&model_data)
        .indices(&indices)
        .transform_data(&transform_data)
        .shader(&shader)
        .atlas(atlas)
        .build()?;
    
    
    
    renderer.clear(0.2, 0.2, 0.2);
    renderer.state()
        .blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
    shader.set_uniform_mat4(renderer.gl(), "u_projection", &projection)?;
    shader.set_uniform_vec2(renderer.gl(), "u_cell_size", metadata.cell_width as f32, metadata.cell_height as f32)?;
    renderer.gl().vertex_attrib1f(2, region as f32);
    renderer.draw(&shader, &vertex_array);

    Ok(())
}
