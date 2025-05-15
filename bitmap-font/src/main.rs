use cosmic_text::{Attrs, BorrowedWithFontSystem, Buffer, Color, Family, FontSystem, Metrics, SwashCache, Weight};
use image::{ImageBuffer, Rgba};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

const PADDING: i32 = 1;
const WHITE: Color = Color::rgb(0xff, 0xff, 0xff);
const GLYPHS: &str = "\
!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnop
qrstuvwxyz{|}~¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãä
åæçèéêëìíîïðñòóôõö÷øùúûüýþÿıƒ‗•←↑→↓↔↕─│┌┐└┘├┤┬┴┼═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬▀▄█
░▒▓ ■□▪▫▬▭▮▯▲▶▼◀◆◇◈◉○◎●◐◑◒◓◕◖◗◢◣◤◥\
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // panic hook
    color_eyre::install()?;
    
    // Create font database
    let mut font_system = FontSystem::new();
    let font_db = font_system.db_mut();

    // Load the font
    font_db.load_font_file("./data/NimbusMonoPS-Regular.otf")?;

    let bitmap_font = BitmapFont::generate(
        &mut font_system,
        GLYPHS,
        24.0,
        60, // grid columns
        6,  // grid rows
    );

    // Save the font files if needed
    bitmap_font.save_texture("./data/bitmap_font.png")?;
    bitmap_font.save_metadata("./data/bitmap_font.json")?;

    println!("Bitmap font generated!");

    Ok(())
}

/// Represents a bitmap font with all its associated metadata
#[derive(Debug)]
pub struct BitmapFont {
    /// The raw RGBA texture data
    pub texture_data: Vec<u32>,
    /// Width of the texture in pixels
    pub texture_width: usize,
    /// Height of the texture in pixels
    pub texture_height: usize,
    /// Width of each character cell
    pub cell_width: usize,
    /// Height of each character cell
    pub cell_height: usize,
    /// Mapping from characters to UV coordinates (u1, v1, u2, v2)
    pub char_to_uv: HashMap<char, (f32, f32, f32, f32)>,
    /// Number of columns in the character grid
    pub grid_cols: usize,
    /// Number of rows in the character grid
    pub grid_rows: usize,
}

impl BitmapFont {
    /// Generate a bitmap font from the provided font, characters, and settings
    pub fn generate(
        font_system: &mut FontSystem,
        chars: &str,
        font_size: f32,
        grid_cols: usize,
        grid_rows: usize
    ) -> Self {
        // set up a metrics object for text layout
        let metrics = Metrics::new(font_size, font_size * 1.2);

        // create a swash cache for rasterization
        let mut swash_cache = SwashCache::new();

        // calculate cell dimensions based on the largest glyph
        let (cell_width, cell_height) = calculate_cell_dimensions(font_system, &mut swash_cache, chars, metrics);
        println!("Cell width: {}", cell_width);
        println!("Cell height: {}", cell_height);

        // calculate the raw texture dimensions based on the grid
        let raw_width = grid_cols * cell_width;
        let raw_height = grid_rows * cell_height;

        // pad to power-of-2 dimensions
        let texture_width = next_pow2(raw_width);
        let texture_height = next_pow2(raw_height);

        // create the texture data (RGBA)
        let mut texture_data = vec![0; texture_width * texture_height];

        // create a mapping of characters to UV coordinates
        let mut char_to_uv = HashMap::new();

        // rasterize each character and place it in the grid
        for (i, c) in chars.chars().enumerate() {
            if i >= grid_cols * grid_rows { break; }

            let grid_x = i % grid_cols;
            let grid_y = i / grid_cols;

            // calculate pixel positions for this cell
            let pixel_x = grid_x * cell_width;
            let pixel_y = grid_y * cell_height;

            // calculate normalized UV coordinates
            let u1 = pixel_x as f32 / texture_width as f32;
            let v1 = pixel_y as f32 / texture_height as f32;
            let u2 = (pixel_x + cell_width) as f32 / texture_width as f32;
            let v2 = (pixel_y + cell_height) as f32 / texture_height as f32;

            // store UV coordinates for this character
            char_to_uv.insert(c, (u1, v1, u2, v2));

            // create a single-character buffer for rasterization
            let mut buffer = Buffer::new(font_system, metrics);
            let mut buffer = buffer.borrow_with(font_system);
            buffer.set_size(2.0 * cell_width as f32, 2.0 * cell_height as f32);


            // add the character to the buffer
            buffer.set_text(&c.to_string(), attrs(), cosmic_text::Shaping::Advanced);
            buffer.shape_until_scroll(true);

            // rasterize the character and place it in the texture
            place_glyph_in_texture(
                &mut swash_cache,
                &mut buffer,
                &mut texture_data,
                texture_width,
                pixel_x as i32,
                pixel_y as i32,
                cell_width as i32,
                cell_height as i32,
            );
        }

        Self {
            texture_data,
            texture_width,
            texture_height,
            cell_width,
            cell_height,
            char_to_uv,
            grid_cols,
            grid_rows,
        }
    }

    /// Save the bitmap font texture as a PNG file
    pub fn save_texture(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
            self.texture_width as u32,
            self.texture_height as u32
        );

        for y in 0..self.texture_height {
            for x in 0..self.texture_width {
                let idx = y * self.texture_width + x;
                if let Some(color) = self.texture_data.get(idx) {
                    let pixel = [
                        (*color >> 24) as u8,
                        (*color >> 16) as u8,
                        (*color >> 8) as u8,
                        *color as u8
                    ];
                    img.put_pixel(x as u32, y as u32, Rgba(pixel));
                    
                }
                if idx < self.texture_data.len() {
                }
            }
        }

        img.save(path)?;
        Ok(())
    }

    /// Save font metadata to a JSON file
    pub fn save_metadata(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let font_info = serde_json::json!({
            "texture_width": self.texture_width,
            "texture_height": self.texture_height,
            "cell_width": self.cell_width,
            "cell_height": self.cell_height,
            "grid_cols": self.grid_cols,
            "grid_rows": self.grid_rows,
            "characters": self.char_to_uv
        });

        let mut file = File::create(path)?;
        Write::write_all(&mut file, serde_json::to_string_pretty(&font_info)?.as_bytes())?;

        Ok(())
    }
}

/// Places a single glyph into the texture at the specified position
fn place_glyph_in_texture(
    swash_cache: &mut SwashCache,
    buffer: &mut BorrowedWithFontSystem<Buffer>,
    texture: &mut [u32],
    texture_width: usize,
    x_offset: i32,
    y_offset: i32,
    width: i32,
    height: i32,
) {
    let texture_width = texture_width as i32;
    
    buffer.draw(swash_cache, WHITE, |x, y, w, h, color| {
        // alpha is non-zero for the glyph pixels
        let a = color.a();
        if a == 0 || x < 0 || x >= width || y < 0 || y >= height || w != 1 || h != 1 {
            return;
        }

        // calculate the pixel position in the texture
        let px = x + x_offset + PADDING;
        let py = y + y_offset + PADDING;
        if px < 0 || py < 0 || px >= texture_width || py >= texture.len() as i32 / texture_width {
            return;
        }

        let idx = (py * texture_width) + px;
        let [r, g, b, a] = color.as_rgba().map(|c| c as u32);
        texture[idx as usize] = r << 24 | g << 16 | b << 8 | a;
    });
    
}

/// Calculates the required cell dimensions for a monospaced bitmap font
/// by finding the maximum width and height of all glyphs in the character set.
fn calculate_cell_dimensions(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    chars: &str,
    metrics: Metrics
) -> (usize, usize) {
    let mut max_width = 0;
    let mut max_height = 0;

    // create a temporary buffer for measuring
    let width = 100.0;  
    let height = 100.0; 

    // iterate through all characters in the set
    for c in chars.chars() {
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        buffer.set_size(width, height);
        
        // add the character to the buffer, then measure it
        buffer.set_text(&c.to_string(), attrs(), cosmic_text::Shaping::Advanced);

        buffer.draw(swash_cache, WHITE, |x, y, _w, _h, color| {
            let a = color.a();
            if a == 0 || x > width as i32 || y > height as i32 {
                return;
            }
            
            max_width = x.max(max_width);
            max_height = y.max(max_height);
        });
    }

    // add some padding
    let cell_width = max_width + PADDING * 2;
    let cell_height = max_height + PADDING * 2;

    (cell_width as _, cell_height as _)
}

/// Rounds up to the next power of 2
fn next_pow2(n: usize) -> usize {
    let mut v = n;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    v
}

fn attrs() -> Attrs<'static> {
    Attrs::new()
        .family(Family::Monospace)
        .weight(Weight::NORMAL)
}