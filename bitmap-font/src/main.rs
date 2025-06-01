mod generator;
mod coordinate;
mod raster_config;
mod grapheme;
mod font_discovery;

use crate::font_discovery::FontDiscovery;
use crate::generator::BitmapFontGenerator;
use font_atlas::*;
use std::fs::File;
use std::io::{self, Write};

const GLYPHS: &str = r#"
!"$#%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnop
qrstuvwxyz{|}~¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãä
åæçèéêëìíîïðñòóôõö÷øùúûüýþÿıƒ‗•←↑→↓↔↕─│┌┐└┘├┤┬┴┼═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬▀▄█
░▒▓ ■□▪▫▲▶▼◀◆◇◈◉○◎●◐◑◒◓◕◖◗◢◣◤◥
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
€₤
😀😃😄😁😆😅🤣😂🙂🙃🫠😉😊😇☺️🥰😍🤩😘😗☺😚😙🥲😋😛😜🤪😝🤑🤗🤭🫢🫣🤫🤔🫡🤐🤨😐😑
😶🫥😶‍🌫️😶‍🌫😏😒🙄😬🤥🫨😮‍💨🙂‍↔️🙂‍↕️😌😔😪🤤😴🫩😷🤒🤕🤢🤮🤧🥵🥶🥴😵🤯😵‍💫🤠🥳🥸😎🤓🧐☹️😕🫤😟
🙁☹😮😯😲😳🥺🥹😦😧😨😰😥😢😭😱😖😣😞😓😩😫🥱😤😡😠🤬😈👿💀☠💩🤡👹👺👻👽👾🤖😺😸
😹😻😼😽🙀😿😾🙈🙉🙊💌💘💝💖💗💓💞💕💟❤‍🔥❤‍🩹❣💔❤🩷🧡💛💚💙🩵💜🤎🖤🩶🤍💋💯💢💥💫💦
💨🕳💬🗨🗯💭💤👋🤚🖐🖐✋🖖🫱🫲🫳🫴🫷🫸👌🤌🤏✌🤞🫰🤟🤘🤙👈👉👆🖕👇☝🫵👍👎✊👊🤛🤜
👏🙌🫶👐🤲🤝🙏✍💅🤳💪🦾🦿🦵🦶👂🦻👃🧠🫀🫁🦷🦴👀👁👅👄🫦👶🧒👦👧🧑👨
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // panic hook
    color_eyre::install()?;

    // parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return Ok(());
    }

    // discover available fonts
    println!("Discovering monospace fonts...");
    let discovery = FontDiscovery::new();
    let available_fonts = discovery.discover_complete_monospace_families();

    if available_fonts.is_empty() {
        eprintln!("No complete monospace font families found!");
        eprintln!("A complete font family must have: Regular, Bold, Italic, and Bold+Italic variants");
        return Ok(());
    }

    println!("\nAvailable monospace fonts with all variants:");
    for (i, font) in available_fonts.iter().enumerate() {
        println!("  {}. {}", i + 1, font.name);
    }

    let selected_font = if args.len() > 1 {
        // try to parse font from command line
        match args[1].parse::<usize>() {
            Ok(idx) if idx > 0 && idx <= available_fonts.len() => {
                &available_fonts[idx - 1]
            }
            _ => {
                // Try to find by name
                available_fonts.iter()
                    .find(|f| f.name.to_lowercase().contains(&args[1].to_lowercase()))
                    .unwrap_or_else(|| {
                        eprintln!("Font '{}' not found, using first available", args[1]);
                        &available_fonts[0]
                    })
            }
        }
    } else { // interactive selection
        println!("\nSelect a font (1-{}) or press Enter for default:", available_fonts.len());
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(idx) = input.trim().parse::<usize>() {
            if idx > 0 && idx <= available_fonts.len() {
                &available_fonts[idx - 1]
            } else {
                println!("Invalid selection, using first font");
                &available_fonts[0]
            }
        } else if input.trim().is_empty() {
            &available_fonts[0]
        } else {
            // Try to find by name
            available_fonts.iter()
                .find(|f| f.name.to_lowercase().contains(&input.trim().to_lowercase()))
                .unwrap_or(&available_fonts[0])
        }
    };

    let font_size = if args.len() > 2 {
        args[2].parse::<f32>().unwrap_or(15.0)
    } else {
        println!("\nEnter font size (default: 15.0):");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().parse::<f32>().unwrap_or(15.0)
    };

    let line_height = if args.len() > 3 {
        args[3].parse::<f32>().unwrap_or(1.2)
    } else {
        println!("\nEnter line height multiplier (default: 1.2):");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().parse::<f32>().unwrap_or(1.2)
    };

    println!("\nGenerating font atlas:");
    println!("  Font: {}", selected_font.name);
    println!("  Size: {}pt", font_size);
    println!("  Line height: {}", line_height);

    // Generate the font
    let bitmap_font = BitmapFontGenerator::new_with_family(
        selected_font.clone(),
        font_size,
        line_height
    )?.generate(GLYPHS);

    bitmap_font.save("./data/bitmap_font.atlas")?;

    println!("\nBitmap font generated!");
    println!("Texture size: {}x{}x{}",
        bitmap_font.atlas_data.texture_width,
        bitmap_font.atlas_data.texture_height,
        bitmap_font.atlas_data.texture_layers);
    println!("Cell size: {}x{}",
        bitmap_font.atlas_data.cell_width,
        bitmap_font.atlas_data.cell_height);
    println!("Total glyph count: {}", bitmap_font.atlas_data.glyphs.len());
    println!("Glyph count per variant: {}/{} (emoji: {})",
        bitmap_font.atlas_data.glyphs.iter().filter(|g| !g.is_emoji).count() / FontStyle::ALL.len(),
        Glyph::GLYPH_ID_MASK + 1, // zero-based id/index
        bitmap_font.atlas_data.glyphs.iter().filter(|g| g.is_emoji).count()
    );
    println!("Longest grapheme in bytes: {}",
        bitmap_font.atlas_data.glyphs.iter()
            .map(|g| g.symbol.len())
            .max()
            .unwrap_or(0)
    );

    Ok(())
}

fn print_help() {
    println!("Bitmap Font Generator");
    println!();
    println!("Usage: bitmap-font [font_name_or_index] [font_size] [line_height]");
    println!();
    println!("Options:");
    println!("  font_name_or_index   Font selection by name (partial match) or index");
    println!("  font_size            Font size in points (default: 15.0)");
    println!("  line_height          Line height multiplier (default: 1.2)");
    println!();
    println!("Examples:");
    println!("  bitmap-font                    # Interactive mode");
    println!("  bitmap-font 1 16 1.5           # Use first font, 16pt, 1.5x line height");
    println!("  bitmap-font \"JetBrains\" 14 1.2 # Find JetBrains font, 14pt");
}

/// Represents a bitmap font with all its associated metadata
#[derive(Debug)]
pub struct BitmapFont {
    /// The properties of the font
    atlas_data: FontAtlasData,
}

impl BitmapFont {
    /// Save bitmap font and metadata to a file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = &self.atlas_data;
        let mut file = File::create(path)?;
        Write::write_all(&mut file, &metadata.to_binary())?;

        Ok(())
    }
}