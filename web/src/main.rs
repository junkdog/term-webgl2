use image::{metadata, DynamicImage, GenericImageView};
use js_sys::wasm_bindgen;
use js_sys::wasm_bindgen::JsCast;
use serde::Deserialize;
use web_sys::console;
use crate::bitmap_font::BitmapFontMetadata;
use crate::error::Error;
use crate::gl::{CellArray, Renderer, Texture, TextureAtlas, GL};
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

// fixme
fn load_image_data(
    image: DynamicImage,
) -> Result<web_sys::ImageData, Error> {
    let (w, h) = image.dimensions();
    let canvas = js::create_canvas(w, h)?;
    let ctx = js::get_2d_context(&canvas)?;

    let rgba_image = image.to_rgba8();
    let raw_data = rgba_image.as_raw();

    // Create an ImageData object from your raw pixels
    let array = wasm_bindgen::Clamped(raw_data.as_slice());
    // array.copy_from(raw_data);

    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        array, w, h
    ).map_err(|_| Error::ImageLoadError("failed to create image data"))?;

    // put the data onto the canvas
    ctx.put_image_data(&image_data, 0.0, 0.0)
        .map_err(|_| Error::ImageLoadError("failed to put image data"))?;

    // Now get it back (usually you'd do transformations in between)
    let result = ctx.get_image_data(0.0, 0.0, w as f64, h as f64)
        .map_err(|_| Error::ImageLoadError("failed to get image data"))?;

    Ok(result)
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
    let vertices: [f32; 16] = [
        //  x      y     u     v
        320.0, 100.0,  1.0,  0.0,  // top-right
        100.0, 560.0,  0.0,  1.0,  // bottom-left
        320.0, 560.0,  1.0,  1.0,  // bottom-right
        100.0, 100.0,  0.0,  0.0,  // top-left
    ];
    // let vertices: [f32; 16] = [
    //     //  x      y    u   v
    //     320.0, 100.0,  u2, v1, //0.25,  1.0,  // top-right
    //     100.0, 560.0,  u1, v2, //0.0,   0.0,  // bottom-left
    //     320.0, 560.0,  u2, v2, //0.25,  0.0,  // bottom-right
    //     100.0, 100.0,  u1, v1, //0.0,   1.0,  // top-left
    // ];

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
    renderer.gl().vertex_attrib1f(2, region as f32);
    renderer.draw(&shader, &vertex_array);

    Ok(())
}
