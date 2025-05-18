use crate::bitmap_font::BitmapFontMetadata;
use crate::error::Error;
use crate::gl::{CellUbo, FontAtlas, Renderer, TerminalCell, TerminalGrid, Texture, GL};
use crate::mat4::Mat4;
use web_sys::console;

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

    let projection = Mat4::orthographic_from_size(
        renderer.canvas_width() as f32,
        renderer.canvas_height() as f32
    );

    // create texture
    const PIXELS: &[u8] = include_bytes!("../../data/bitmap_font.png");
    const METADATA_JSON: &'static str = include_str!("../../data/bitmap_font.json");
    
    let metadata: BitmapFontMetadata = BitmapFontMetadata::from_json(METADATA_JSON)?;
    let texture = Texture::from_image_data(renderer.gl(), GL::RGBA, PIXELS, &metadata)?;
    let atlas = FontAtlas::from_bitmap_font(renderer.gl(), texture, &metadata)?;

    let region = atlas.get_glyph_depth("B").unwrap();
    console::log_1(&format!("{:?}", region).into());
    // model data
    let cell_width = metadata.cell_width  ; // - BitmapFontMetadata::PADDING * 2;
    let cell_height = metadata.cell_height; // - BitmapFontMetadata::PADDING * 2;
    let (w, h) = (cell_width as f32, cell_height as f32);
    let model_data: [f32; 16] = [
        //  x      y     u     v
            w,   0.0,  1.0,  0.0,  // top-right
          0.0,     h,  0.0,  1.0,  // bottom-left
            w,     h,  1.0,  1.0,  // bottom-right
          0.0,   0.0,  0.0,  0.0,  // top-left
    ];

    let transform_data = crate_cell_instance_data(
        &atlas,
        (renderer.canvas_width(), renderer.canvas_height()),
        &metadata,
    );

    let indices = [
        0, 1, 2, // first triangle
        0, 3, 1, // second triangle
    ];

    let terminal_grid = TerminalGrid::builder()
        .gl(renderer.gl())
        .model_data(&model_data)
        .indices(&indices)
        .transform_data(&transform_data)
        .atlas(atlas)
        .build()?;
    
    terminal_grid.upload_ubo_data(renderer.gl(), CellUbo {
        projection: projection.data,
        cell_size: [cell_width as f32, cell_height as f32],
    });
    
    renderer.begin_frame();
    renderer.render(&terminal_grid);
    renderer.end_frame();

    Ok(())
}

fn crate_cell_instance_data(
    font_atlas: &FontAtlas,
    screen_size: (i32, i32),
    metadata: &BitmapFontMetadata,
) -> Vec<TerminalCell> {
    let (cell_width, cell_height) = (metadata.cell_width, metadata.cell_height);
    let (cols, rows) = (screen_size.0 / cell_width, screen_size.1 / cell_height);

    let mut instance_data = Vec::new();

    let mut rng = SimpleRng::default();

    let s = "hello, ratatui/ratzilla! ";

    for row in 0..rows {
        for col in 0..cols {
            let depth = (row * cols + col) % metadata.char_to_uv.len() as i32;
            let (a, b) = ((col as usize) % s.len(), (col as usize + 1) % s.len());
            let (a, b) = if a > b {
                (0, 1)
            } else {
                (a, b)
            };

            let depth = font_atlas.get_glyph_depth(&s[a..b]).unwrap_or_else(|| {
                console::error_1(&format!("Failed to get depth for char: {}", &s[a..=b]).into());
                0
            });
            let fg = rng.gen() | 0xff;
            let bg = rng.gen() | 0xff;
            let fg = 0xffffffff;
            // let bg = 0x000000ff;
            instance_data.push(TerminalCell::new((col as u16, row as u16), depth as u16, fg, bg));
        }
    }

    instance_data
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
