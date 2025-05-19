use cosmic_text::{Attrs, BorrowedWithFontSystem, Buffer, Color, Family, FontSystem, Metrics, SwashCache, Weight};
use image::{ImageBuffer, Rgba};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use font_atlas::*;

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
        16.0,
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
    /// The properties of the font
    metadata: FontAtlasConfig,
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
        let (cell_w, cell_h) = calculate_cell_dimensions(font_system, &mut swash_cache, chars, metrics);
        println!("Cell width: {}", cell_w);
        println!("Cell height: {}", cell_h);

        // calculate the raw texture dimensions based on the grid
        let raw_width = grid_cols * cell_w as usize;
        let raw_height = grid_rows * cell_h as usize;

        // pad to power-of-2 dimensions
        let texture_width = next_pow2(raw_width);
        let texture_height = next_pow2(raw_height);

        // create the texture data (RGBA)
        let mut texture_data = vec![0; texture_width * texture_height];

        let mut glyphs = Vec::new();

        // rasterize each character and place it in the grid
        for (i, c) in chars.chars().enumerate() {
            if i >= grid_cols * grid_rows { break; }

            let grid_x = (i % grid_cols) as i32;
            let grid_y = (i / grid_cols) as i32;

            // calculate pixel positions for this cell
            let pixel_x = grid_x * cell_w;
            let pixel_y = grid_y * cell_h;


            // store id and pixel coordinates for this glyph
            glyphs.push(Glyph::new(c, (pixel_x + PADDING, pixel_y + PADDING)));

            // create a single-character buffer for rasterization
            let mut buffer = Buffer::new(font_system, metrics);
            let mut buffer = buffer.borrow_with(font_system);
            buffer.set_size(2.0 * cell_w as f32, 2.0 * cell_h as f32);

            // add the character to the buffer
            buffer.set_text(&c.to_string(), attrs(), cosmic_text::Shaping::Advanced);
            buffer.shape_until_scroll(true);

            // rasterize the character and place it in the texture
            place_glyph_in_texture(
                &mut swash_cache,
                &mut buffer,
                &mut texture_data,
                texture_width,
                pixel_x,
                pixel_y,
                cell_w,
                cell_h,
            );
        }

        assign_missing_glyph_ids(&mut glyphs);

        Self {
            texture_data,
            metadata: FontAtlasConfig {
                font_size,
                texture_width,
                texture_height,
                cell_width: cell_w,
                cell_height: cell_h,
                glyphs
            },
        }
    }

    /// Save the bitmap font texture as a PNG file
    pub fn save_texture(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
            self.metadata.texture_width as u32,
            self.metadata.texture_height as u32
        );

        for y in 0..self.metadata.texture_height {
            for x in 0..self.metadata.texture_width {
                let idx = y * self.metadata.texture_width + x;
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
        let metadata = &self.metadata;
        let font_info = serde_json::json!({
            "font_size": metadata.font_size,
            "texture_width": metadata.texture_width,
            "texture_height": metadata.texture_height,
            "cell_width": metadata.cell_width,
            "cell_height": metadata.cell_height,
            "glyphs": metadata.glyphs.iter().map(|g| {
                serde_json::json!({
                    "id": g.id,
                    "symbol": g.symbol,
                    "pixel_coords": g.pixel_coords
                })
            }).collect::<Vec<_>>()
        });

        let mut file = File::create(path)?;
        Write::write_all(&mut file, serde_json::to_string_pretty(&font_info)?.as_bytes())?;

        Ok(())
    }
}

fn assign_missing_glyph_ids(glyphs: &mut [Glyph]) {
    // pre-assigned glyphs (in the range 0x0000-0x00FF)
    let mut used_ids = HashSet::new();
    glyphs.iter()
        .filter(|g| g.id != 0xFFFF)
        .for_each(|g| {
            used_ids.insert(g.id);
        });

    let mut next_id: i32 = -1; // initial value to -1
    let mut next_glyph_id = || {
        let mut id = next_id;
        while id == -1 || used_ids.contains(&(id as u16)) {
            id += 1;
        }

        next_id = id + 1;
        id as u16
    };

    for g in glyphs.iter_mut().filter(|g| g.id == 0xFFFF) {
        g.id = next_glyph_id();
    }

    // sort the glyphs by their ID
    glyphs.sort_by(|a, b| a.id.cmp(&b.id));
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
) -> (i32, i32) {
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

    (cell_width, cell_height)
}

// Rounds up to the next power of 2
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_pow2() {
        assert_eq!(next_pow2(0), 1);
        assert_eq!(next_pow2(1), 1);
        assert_eq!(next_pow2(2), 2);
        assert_eq!(next_pow2(3), 4);
        assert_eq!(next_pow2(4), 4);
        assert_eq!(next_pow2(5), 8);
        assert_eq!(next_pow2(15), 16);
        assert_eq!(next_pow2(16), 16);
        assert_eq!(next_pow2(17), 32);
        assert_eq!(next_pow2(1023), 1024);
    }
}