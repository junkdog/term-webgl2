use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use fontdue::{Font, FontSettings};


const GLYPHS: &str = "\
!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnop
qrstuvwxyz{|}~¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãä
åæçèéêëìíîïðñòóôõö÷øùúûüýþÿıƒ‗•←↑→↓↔↕─│┌┐└┘├┤┬┴┼═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬▀▄█
░▒▓ ■□▪▫▬▭▮▯▲▶▼◀◆◇◈◉○◎●◐◑◒◓◕◖◗◢◣◤◥\
";


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font_data = std::fs::read("data/NimbusMonoPS-Regular.otf")?;
    let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();

    let bitmap_font = BitmapFont::generate(
        &font,
        GLYPHS,
        24.0,
        50, // grid columns
        6,  // grid rows
    );

    // Save the font files if needed
    bitmap_font.save_texture("data/bitmap_font.png")?;
    bitmap_font.save_metadata("data/bitmap_font.json")?;

    println!("Bitmap font generated!");

    Ok(())
}

/// Represents a bitmap font with all its associated metadata
#[derive(Debug)]
pub struct BitmapFont {
    /// The raw RGBA texture data
    pub texture_data: Vec<u8>,
    /// Width of the texture in pixels
    pub texture_width: usize,
    /// Height of the texture in pixels
    pub texture_height: usize,
    /// Width of each character cell
    pub cell_width: usize,
    /// Height of each character cell
    pub cell_height: usize,
    /// Mapping from characters to UV coordinates (u1, v1, u2, v2)
    pub char_to_uv: std::collections::HashMap<char, (f32, f32, f32, f32)>,
    /// Number of columns in the character grid
    pub grid_cols: usize,
    /// Number of rows in the character grid
    pub grid_rows: usize,
}

impl BitmapFont {
    /// Generate a bitmap font from the provided font, characters, and settings
    pub fn generate(
        font: &Font,
        chars: &str,
        font_size: f32,
        grid_cols: usize,
        grid_rows: usize
    ) -> Self {
        // calculate cell dimensions based on the largest glyph
        let (cell_width, cell_height) = calculate_cell_dimensions(font, chars, font_size);

        // calculate the raw texture dimensions based on the grid
        let raw_width = grid_cols * cell_width;
        let raw_height = grid_rows * cell_height;

        // pad to power-of-2 dimensions
        let texture_width = next_pow2(raw_width);
        let texture_height = next_pow2(raw_height);

        // create the texture data (RGBA)
        let mut texture_data = vec![0; texture_width * texture_height * 4];

        // create a mapping of characters to UV coordinates
        let mut char_to_uv = std::collections::HashMap::new();

        // rasterize each character and place it in the grid
        for (i, c) in chars.chars().enumerate() {
            if i >= grid_cols * grid_rows { break; }

            let grid_x = i % grid_cols;
            let grid_y = i / grid_cols;

            let (metrics, bitmap) = font.rasterize(c, font_size);

            // calculate pixel positions
            let pixel_x = grid_x * cell_width;
            let pixel_y = grid_y * cell_height;

            // calculate normalized UV coordinates
            let u1 = pixel_x as f32 / texture_width as f32;
            let v1 = pixel_y as f32 / texture_height as f32;
            let u2 = (pixel_x + cell_width) as f32 / texture_width as f32;
            let v2 = (pixel_y + cell_height) as f32 / texture_height as f32;

            // store UV coordinates for this character
            char_to_uv.insert(c, (u1, v1, u2, v2));

            // place the glyph in the texture
            place_glyph_in_texture(
                &bitmap,
                metrics.width,
                metrics.height,
                &mut texture_data,
                texture_width,
                pixel_x.saturating_add(metrics.xmin.abs() as usize),
                pixel_y.saturating_sub(metrics.ymin.abs() as usize),
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
        use image::{ImageBuffer, Rgba};

        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
            self.texture_width as u32,
            self.texture_height as u32
        );

        for y in 0..self.texture_height {
            for x in 0..self.texture_width {
                let idx = (y * self.texture_width + x) * 4;
                if idx + 3 < self.texture_data.len() {
                    let pixel = Rgba([
                        self.texture_data[idx],     // R
                        self.texture_data[idx + 1], // G
                        self.texture_data[idx + 2], // B
                        self.texture_data[idx + 3], // A
                    ]);
                    img.put_pixel(x as u32, y as u32, pixel);
                }
            }
        }

        img.save(path)?;
        Ok(())
    }

    /// Save font metadata to a JSON file
    pub fn save_metadata(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let font_info = serde_json::json!({
            "textureWidth": self.texture_width,
            "textureHeight": self.texture_height,
            "cellWidth": self.cell_width,
            "cellHeight": self.cell_height,
            "gridCols": self.grid_cols,
            "gridRows": self.grid_rows,
            "characters": self.char_to_uv
        });

        let mut file = File::create(path)?;
        Write::write_all(&mut file, serde_json::to_string_pretty(&font_info)?.as_bytes())?;

        Ok(())
    }
}

/// Places a single glyph bitmap into the texture at the specified position
fn place_glyph_in_texture(
    bitmap: &[u8],          // The rasterized glyph bitmap (grayscale)
    glyph_width: usize,     // Width of the glyph
    glyph_height: usize,    // Height of the glyph
    texture: &mut [u8],     // The texture data (RGBA)
    texture_width: usize,   // Width of the texture
    x_offset: usize,        // X position in the texture
    y_offset: usize,        // Y position in the texture
) {
    // add padding to center the glyph in its cell
    let padding = 1;
    let x_offset = x_offset + padding;
    let y_offset = y_offset + padding;

    // copy each pixel of the glyph to the texture
    for y in 0..glyph_height {
        for x in 0..glyph_width {
            let glyph_idx = y * glyph_width + x;

            // skip if out of bounds (safety check)
            if glyph_idx >= bitmap.len() {
                continue;
            }

            // the glyph bitmap is grayscale (1 byte per pixel)
            let alpha = bitmap[glyph_idx];

            // calculate the position in the texture (RGBA, 4 bytes per pixel)
            let texture_x = x_offset + x;
            let texture_y = y_offset + y;
            let texture_idx = (texture_y * texture_width + texture_x) * 4;

            // skip if out of bounds of the texture
            if texture_idx + 3 >= texture.len() {
                continue;
            }

            // set RGBA values - white with varying alpha
            texture[texture_idx] = 255;       // R
            texture[texture_idx + 1] = 255;   // G
            texture[texture_idx + 2] = 255;   // B
            texture[texture_idx + 3] = alpha; // A
        }
    }
}


/// Calculates the required cell dimensions for a monospaced bitmap font
/// by finding the maximum width and height of all glyphs in the character set.
fn calculate_cell_dimensions(font: &fontdue::Font, chars: &str, size: f32) -> (usize, usize) {
    let mut max_width = 0;
    let mut max_height = 0;

    // iterate through all characters in the set
    for c in chars.chars() {
        // get the metrics for this character
        let (metrics, _) = font.rasterize(c, size);

        // update maximums if this character is larger
        max_width = max_width.max(metrics.width);
        max_height = max_height.max(metrics.height);

        // Also consider the advance width to ensure proper spacing
        let advance_width = metrics.advance_width.ceil() as usize;
        max_width = max_width.max(advance_width);
    }

    // Add some padding to avoid characters being too close to the cell edges
    // This helps prevent bleeding between adjacent glyphs when rendering
    let padding = 1;
    let cell_width = max_width + padding * 2;
    let cell_height = max_height + padding * 2;

    (cell_width, cell_height)
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