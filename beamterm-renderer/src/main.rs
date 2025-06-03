use web_sys::console;
use beamterm_data::FontAtlasData;
use beamterm_renderer::DEFAULT_FONT_ATLAS_BLOB;
use crate::error::Error;
use crate::gl::{FontAtlas, Renderer, TerminalGrid};

mod gl;
mod error;
mod mat4;
mod js;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run().unwrap()
}

fn run() -> Result<(), Error> {
    
    let mut renderer = Renderer::create("canvas")?;
    let gl = renderer.gl();

    let atlas_data: FontAtlasData = FontAtlasData::from_binary(DEFAULT_FONT_ATLAS_BLOB)
        .map_err(|e| Error::deserialization_failed(e.message))?;
    
    console::log_1(&format!("Font Atlas: {:?}", atlas_data).into());
    
    let atlas = FontAtlas::load(gl, atlas_data)?;

    let canvas_size = renderer.canvas_size();
    let terminal_grid = TerminalGrid::new(gl, atlas, canvas_size)?;

    renderer.begin_frame();
    renderer.render(&terminal_grid);
    renderer.end_frame();

    Ok(())
}

