use font_atlas::FontAtlasConfig;
use webgl2::{BITMAP_FONT_IMAGE, BITMAP_FONT_METADATA};
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

    let font_config: FontAtlasConfig = FontAtlasConfig::from_binary(BITMAP_FONT_METADATA)
        .map_err(|e| Error::JsonDeserializationError(e.message))?;
    
    let cell_size = font_config.cell_size();
    let atlas = FontAtlas::load(gl, BITMAP_FONT_IMAGE, font_config)?;

    let canvas_size = renderer.canvas_size();
    let terminal_grid = TerminalGrid::new(gl, atlas, canvas_size)?;
    terminal_grid.upload_ubo_data(gl, canvas_size, cell_size);

    renderer.begin_frame();
    renderer.render(&terminal_grid);
    renderer.end_frame();

    Ok(())
}

