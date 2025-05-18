use crate::bitmap_font::FontAtlasConfig;
use crate::error::Error;
use crate::gl::{CellUbo, FontAtlas, Renderer, TerminalGrid};
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

    const PIXELS: &[u8] = include_bytes!("../../data/bitmap_font.png");
    const METADATA_JSON: &'static str = include_str!("../../data/bitmap_font.json");
    
    let font_config: FontAtlasConfig = FontAtlasConfig::from_json(METADATA_JSON)?;
    let atlas = FontAtlas::load(renderer.gl(), PIXELS, &font_config)?;

    let terminal_grid = TerminalGrid::builder()
        .gl(renderer.gl())
        .screen_size(renderer.canvas_size())
        .atlas(atlas)
        .font_config(&font_config)
        .build()?;
    
    let (cell_width, cell_height) = font_config.cell_size();
    terminal_grid.upload_ubo_data(renderer.gl(), CellUbo {
        projection: Mat4::orthographic_from_size(
            renderer.canvas_width() as f32,
            renderer.canvas_height() as f32
        ).data,
        cell_size: [cell_width as f32, cell_height as f32],
    });
    
    renderer.begin_frame();
    renderer.render(&terminal_grid);
    renderer.end_frame();

    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    const A: u32 = 1664525;
    const C: u32 = 1013904223;

    pub fn new(seed: u32) -> Self {
        SimpleRng { state: seed }
    }

    pub fn gen(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }
}

impl Default for SimpleRng {
    fn default() -> Self {
        let seed = web_time::SystemTime::now()
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32;

        SimpleRng::new(seed)
    }
}
