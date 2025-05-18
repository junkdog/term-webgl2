use crate::font_atlas::FontAtlasConfig;
use crate::error::Error;
use crate::gl::{FontAtlas, Renderer, TerminalGrid};

mod gl;
mod error;
mod mat4;
mod font_atlas;
mod js;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run().unwrap()
}

fn run() -> Result<(), Error> {
    
    let mut renderer = Renderer::create("canvas")?;
    let gl = renderer.gl();

    const PIXELS: &[u8] = include_bytes!("../../data/bitmap_font.png");
    const METADATA_JSON: &'static str = include_str!("../../data/bitmap_font.json");

    let font_config: FontAtlasConfig = FontAtlasConfig::from_json(METADATA_JSON)?;
    let atlas = FontAtlas::load(gl, PIXELS, &font_config)?;

    let (canvas_size, cell_size) = (renderer.canvas_size(), font_config.cell_size());
    let terminal_grid = TerminalGrid::new(gl, atlas, canvas_size, cell_size)?;
    terminal_grid.upload_ubo_data(gl, canvas_size, cell_size);

    renderer.begin_frame();
    renderer.render(&terminal_grid);
    renderer.end_frame();

    Ok(())
}
