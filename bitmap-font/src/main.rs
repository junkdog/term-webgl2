mod generator;

use crate::generator::BitmapFontGenerator;
use font_atlas::*;
use image::{ImageBuffer, Rgba};
use std::fs::File;
use std::io::Write;
use std::process::exit;

const PADDING: i32 = 1;
const GLYPHS: &str = "
!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnop
qrstuvwxyz{|}~¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãä
åæçèéêëìíîïðñòóôõö÷øùúûüýþÿıƒ‗•←↑→↓↔↕─│┌┐└┘├┤┬┴┼═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬▀▄█
░▒▓ ■□▪▫▬▭▮▯▲▶▼◀◆◇◈◉○◎●◐◑◒◓◕◖◗◢◣◤◥
├─└─│─┤─┬─┴─┼─┌─┐─╶╴╷╵╸╺╻╹
∀∃∄∅∆∇∈∉∋∌∏∑∞∟∠∡∢∥∧∨∩∪∫∮
≈≠≡≤≥≦≧≨≩≪≫≬≭≮≯≰≱≲≳≴≵≶≷≸≹≺≻≼≽≾≿
➜➤➥➦➧➨➩➪➫➬➭➮➯➱➲➳➴➵➶➷➸➹➺➻➼➽➾
◊◈◉○◎●◐◑◒◓◔◕◖◗◢◣◤◥▲▶▼◀◆◇▁▂▃▄▅▆▇█▓▒░▒▓█
⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟
⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿
⡀⡁⡂⡃⡄⡅⡆⡇⡈⡉⡊⡋⡌⡍⡎⡏⡐⡑⡒⡓⡔⡕⡖⡗⡘⡙⡚⡛⡜⡝⡞⡟
⡠⡡⡢⡣⡤⡥⡦⡧⡨⡩⡪⡫⡬⡭⡮⡯⡰⡱⡲⡳⡴⡵⡶⡷⡸⡹⡺⡻⡼⡽⡾⡿
◐◑◒◓◔◕◖◗⊙⏴⏵⏶⏷▶
";

const EMOJI_GLYPHS: &str = "
➰⌚⏰⏱⏲⏳⏸⏹⏺⏯⏮⏭
";


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // panic hook
    color_eyre::install()?;
    
    let bitmap_font = BitmapFontGenerator::new(10.0, 1024)
        .generate(GLYPHS);

    // Save the font files if needed
    // bitmap_font.save_texture("./data/bitmap_font.png")?;
    // bitmap_font.save_metadata("./data/bitmap_font.atlas")?;
    bitmap_font.save("./data/bitmap_font.atlas")?;
    
    println!("Bitmap font generated!");
    println!("Texture size: {}x{}", bitmap_font.atlas_data.texture_width, bitmap_font.atlas_data.texture_height);
    println!("Cell size: {}x{}", bitmap_font.atlas_data.cell_width, bitmap_font.atlas_data.cell_height);
    println!("Total glyph count: {}", bitmap_font.atlas_data.glyphs.len());
    println!("Glyph count per variant: {}/{}", 
        bitmap_font.atlas_data.glyphs.len() / FontStyle::ALL.len(),
        Glyph::GLYPH_ID_MASK + 1 // zero-based id/index
    );
    
    Ok(())
}


/// Represents a bitmap font with all its associated metadata
#[derive(Debug)]
pub struct BitmapFont {
    /// The properties of the font
    atlas_data: FontAtlasData,
}

impl BitmapFont {
    /// Generate a bitmap font from the provided font, characters, and settings
    pub fn generate(
        chars: &str,
        font_size: f32,
        texture_width: usize,
    ) -> Self {
        BitmapFontGenerator::new(font_size, texture_width)
            .generate(chars)
    }

    /// Save the bitmap font texture as a PNG file
    // pub fn save_texture(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
    //         self.metadata.texture_width,
    //         self.metadata.texture_height
    //     );
    // 
    //     for y in 0..self.metadata.texture_height {
    //         for x in 0..self.metadata.texture_width {
    //             let idx = y * self.metadata.texture_width + x;
    //             if let Some(color) = self.texture_data.get(idx as usize) {
    //                 let pixel = [
    //                     (*color >> 24) as u8,
    //                     (*color >> 16) as u8,
    //                     (*color >> 8) as u8,
    //                     *color as u8
    //                 ];
    //                 img.put_pixel(x, y, Rgba(pixel));
    //                 
    //             }
    //         }
    //     }
    // 
    //     img.save(path)?;
    //     Ok(())
    // }

    // fn save_metadata(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     let metadata = &self.atlas_data;
    //     let mut file = File::create(path)?;
    //     Write::write_all(&mut file, &metadata.to_binary())?;
    // 
    //     Ok(())
    // }
    
    /// Save bitmap font and metadata to a file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = &self.atlas_data;
        let mut file = File::create(path)?;
        Write::write_all(&mut file, &metadata.to_binary())?;
        
        Ok(())
    }
}
