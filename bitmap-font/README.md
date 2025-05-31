# bitmap-font

A font atlas generator for WebGL terminal renderers, optimized for GPU texture memory and
rendering efficiency.

## Overview

`bitmap-font` generates tightly-packed 3D texture atlases from TTF/OTF font files, producing a
binary format optimized for GPU upload. The system supports multiple font styles, full Unicode
including emoji, and automatic grapheme clustering.

## Architecture

The crate consists of:
- **Font rasterization engine** using cosmic-text for high-quality text rendering
- **3D texture packer** organizing glyphs into 4×4 grids per texture slice
- **Binary serializer** with zlib compression for efficient storage
- **Atlas verification tool** for debugging and visualization

## Glyph ID Assignment System

### ID Structure

The system uses a 16-bit glyph ID that encodes both the base character and its style variations:

| Bit Range | Purpose       | Description                            |
|-----------|---------------|----------------------------------------|
| 0-8       | Base Glyph ID | 512 possible base glyphs (0x000-0x1FF) |
| 9         | Bold Flag     | Selects bold variant (0x0200)          |
| 10        | Italic Flag   | Selects italic variant (0x0400)        |
| 11        | Emoji Flag    | Indicates emoji glyph (0x0800)         |
| 12-15     | N/A           |                                        |

### Font Style Encoding

Each base glyph automatically generates four style variants by combining the bold and italic flags:

| Style       | Bit Pattern | ID Offset | Example ('A' = 0x41) |
|-------------|-------------|-----------|----------------------|
| Normal      | `0x0000`    | +0        | `0x0041`             |
| Bold        | `0x0200`    | +512      | `0x0241`             |
| Italic      | `0x0400`    | +1024     | `0x0441`             |
| Bold+Italic | `0x0600`    | +1536     | `0x0641`             |

This encoding allows the shader to compute texture coordinates directly from the glyph ID without
lookup tables.

### Character Category Assignment

The generator assigns IDs based on three character categories:

**1. ASCII Characters (0x00-0x7F)**
- Direct mapping: character code = base glyph ID
- Guarantees fast lookup for common characters
- Occupies first 8 texture slices (128 chars ÷ 16 per slice)

**2. Unicode Characters**
- Fill unused slots in the 0x00-0x1FF range
- Sequential assignment starting from first available ID
- Maximum 384 additional non-ASCII glyphs (512 total - 128 ASCII)

**3. Emoji Characters**
- Start at ID 0x800 (bit 11 set)
- Sequential assignment: 0x800, 0x801, 0x802...
- No style variants (emoji are always rendered as-is)
- Can extend beyond the 512 base glyph limit

### Texture Slice Calculation

With the ID assignment scheme:
- Regular glyphs with styles: IDs 0x0000-0x07FF (first 128 slices)
- Emoji glyphs: IDs 0x0800+ (slices 128+)

For a typical atlas with ~500 base glyphs + 100 emoji:
- Base glyphs × 4 styles = 2000 IDs → 125 slices
- Emoji = 100 IDs → 7 additional slices
- Total ≈ 132 slices → rounded to 256 (next power of 2)

## 3D Texture Organization

### Slice Layout

Each texture slice contains a 4×4 grid of glyphs:

```
Position in slice = ID & 0x0F (modulo 16)
Grid X = Position % 4
Grid Y = Position ÷ 4
Slice Z = ID ÷ 16
```

### Memory Layout

The 3D texture uses RGBA format with dimensions:
- Width: cell_width × 4
- Height: cell_height × 4
- Depth: next_power_of_2(max_glyph_id ÷ 16)

The RGBA format is required for emoji support - while monochrome glyphs could use a single channel,
emoji glyphs need full color information.

This layout ensures:
- Efficient GPU memory alignment
- Cache-friendly access patterns (related glyphs in same slice)
- Simple coordinate calculation using bit operations

## Rasterization Process

### Cell Dimension Calculation

The system determines cell size by measuring the full block character `█` to ensure all glyphs
fit within the cell boundaries. Additional padding of 1px on all sides prevents texture bleeding.

### Font Style Handling

Each glyph is rendered four times with appropriate font selection based on the style flags.

### Emoji Special Handling

Emoji glyphs require special processing:
1. Rendered at 2× size for measurement
2. Scaled down to fit within cell boundaries
3. Centered within the cell
4. Color information preserved in texture

The presence of emoji is the primary reason the atlas uses RGBA format instead of a single-channel
texture. While monochrome glyphs only need an alpha channel, emoji require full color information
to render correctly.

## Binary Atlas Format

### File Structure

The atlas uses a versioned binary format with header validation:

```
Header (5 bytes)
├─ Magic: [0xBA, 0xB1, 0xF0, 0xA5]
└─ Version: 0x01

Metadata Section
├─ Font size (f32)
├─ Texture dimensions (u32 × 3)
├─ Cell dimensions (i32 × 2)
└─ Glyph count (u16)

Glyph Definitions
└─ Per glyph:
   ├─ ID (u16)
   ├─ Style (u8)
   ├─ Is emoji (u8)
   ├─ Pixel coordinates (i32 × 2)
   └─ Symbol (length-prefixed UTF-8)

Compressed Texture Data
└─ zlib-compressed RGBA data
```

### Serialization Properties

- **Endianness**: Little-endian for cross-platform compatibility
- **Compression**: zlib level 9 (typically 75% size reduction)
- **String encoding**: Length-prefixed UTF-8 (max 255 bytes)
- **Alignment**: Natural alignment without padding

## Usage

### Generation

The tool generates an atlas from a predefined character set including:
- Full ASCII and Latin-1 supplement
- Box drawing characters
- Mathematical symbols
- Arrows and geometric shapes
- Braille patterns
- Extensive emoji set

### Verification

The `verify-atlas` binary visualizes the texture layout, showing:
- Slice organization
- Character placement
- Grid boundaries
- Glyph distribution

