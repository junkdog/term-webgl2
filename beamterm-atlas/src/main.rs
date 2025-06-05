mod cli;
mod coordinate;
mod font_discovery;
mod generator;
mod grapheme;
mod raster_config;

use std::{fs::File, io::Write};

use beamterm_data::*;
use clap::Parser;

use crate::{cli::Cli, font_discovery::FontDiscovery, generator::BitmapFontGenerator};

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
    let cli = Cli::parse();

    // handle --list-fonts flag
    if cli.list_fonts {
        Cli::display_font_list();
        return Ok(());
    }

    // validate CLI arguments
    cli.validate()?;

    // discover available fonts
    let discovery = FontDiscovery::new();
    let available_fonts = discovery.discover_complete_monospace_families();

    if available_fonts.is_empty() {
        eprintln!("No complete monospace font families found!");
        eprintln!(
            "A complete font family must have: Regular, Bold, Italic, and Bold+Italic variants"
        );
        return Ok(());
    }

    // select font
    let selected_font = cli.select_font(&available_fonts)?;

    // print configuration summary
    cli.print_summary(&selected_font.name);

    // TODO: Pass underline/strikethrough configuration to the generator
    // These parameters should be stored in FontAtlasData for use during rendering
    // Currently, the shader uses hardcoded values for these effects
    let underline = LineDecoration::new(cli.underline_position, cli.underline_thickness / 100.0);
    let strikethrough =
        LineDecoration::new(cli.strikethrough_position, cli.strikethrough_thickness / 100.0);

    // Generate the font
    let bitmap_font = BitmapFontGenerator::new_with_family(
        selected_font.clone(),
        cli.font_size,
        cli.line_height,
        underline,
        strikethrough,
    )?
    .generate(GLYPHS);

    bitmap_font.save(&cli.output)?;

    let atlas = &bitmap_font.atlas_data;
    println!("\nBitmap font generated!");
    println!(
        "Texture size: {}x{}x{}",
        atlas.texture_dimensions.0, atlas.texture_dimensions.1, atlas.texture_dimensions.2
    );
    println!(
        "Cell size: {}x{}",
        bitmap_font.atlas_data.cell_size.0, bitmap_font.atlas_data.cell_size.1
    );
    println!("Total glyph count: {}", bitmap_font.atlas_data.glyphs.len());
    println!(
        "Glyph count per variant: {}/{} (emoji: {})",
        bitmap_font.atlas_data.glyphs.iter().filter(|g| !g.is_emoji).count() / FontStyle::ALL.len(),
        Glyph::GLYPH_ID_MASK + 1, // zero-based id/index
        bitmap_font.atlas_data.glyphs.iter().filter(|g| g.is_emoji).count()
    );
    println!(
        "Longest grapheme in bytes: {}",
        bitmap_font.atlas_data.glyphs.iter().map(|g| g.symbol.len()).max().unwrap_or(0)
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
    /// Save bitmap font and metadata to a file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = &self.atlas_data;
        let mut file = File::create(path)?;
        Write::write_all(&mut file, &metadata.to_binary())?;

        Ok(())
    }
}
