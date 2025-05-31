## WebGL2 Terminal Renderer

A high-performance terminal rendering system for web browsers, targeting sub-millisecond render times.

## Key Features

- **Single Draw Call** - Renders entire terminal (e.g., 200×80 cells) in one instanced draw
- **Zero-Copy Updates** - Direct memory mapping for dynamic cell updates
- **Unicode and Emoji Support** - Complete Unicode support with grapheme clustering
- **ASCII Fast Path** - Direct bit operations for ASCII characters (no lookups)


## Performance

For a typical 12×18 pixel font with ~2500 glyphs:

| Metric                          | Value                   |
|---------------------------------|-------------------------|
| Render Time                     | <1ms for 16,000 cells   |
| Draw Calls                      | 1 per frame             |
| Memory Usage                    | ~3.5MB total GPU memory |
| Update Bandwidth (full refresh) | ~8 MB/s @ 60 FPS        |


## System Architecture

The renderer consists of three specialized crates:

**`bitmap-font`** - Generates GPU-optimized font atlases from TTF/OTF files. Automatically calculates
cell dimensions, supports font styles (normal/bold/italic), and outputs packed texture data.

**`font-atlas`** - Provides shared data structures and efficient binary serialization. Features
versioned format with header validation and cross-platform encoding.

**`term-renderer`** - The WebGL2 rendering engine. Implements instanced rendering with optimized
buffer management and state tracking for consistent sub-millisecond performance.


## Architecture Overview

The architecture leverages GPU instancing to reuse a single quad geometry across all terminal cells,
with per-instance data providing position, character, and color information. The 3D texture atlas
maximizes cache efficiency by packing related glyphs into 4×4 grids within each texture slice.

### Buffer Management Strategy

The renderer employs several optimization strategies:

1. **VAO Encapsulation**: All vertex state is captured in a single VAO, minimizing state changes
2. **Separate Static/Dynamic**: Geometry and positions rarely change; only cell content is dynamic
3. **Aligned Packing**: All structures use explicit alignment for optimal GPU access
4. **Batch Updates**: Cell updates are batched and uploaded in a single operation
5. **Immutable Storage**: 3D texture uses `texStorage3D` for driver optimization hints

These strategies combined enable the renderer to achieve consistent sub-millisecond frame times even
for large terminals (200×80 cells = 16,000 instances).

### Total Memory Requirements

For a 200×80 terminal with 2048 glyphs:

| Component      | Size      | Type                        |
|----------------|-----------|-----------------------------|
| Font Atlas     | 1.7 MB    | Texture memory              |
| Static Buffers | 64 KB     | Vertex + Instance positions |
| Dynamic Buffer | 128 KB    | Cell content                |
| Overhead       | ~10 KB    | VAO, shaders, uniforms      |
| **Total**      | **~2 MB** | GPU memory                  |

This efficient memory usage allows multiple terminal instances without significant GPU memory 
pressure.

## Terminal Grid Renderer API

The WebGL2 terminal renderer provides efficient text rendering through a simple API centered
around two main components: `TerminalGrid` for managing the display and `FontAtlas` for glyph storage.

### Quick Start

The terminal renderer provides a high-performance WebGL2-based text rendering system:

```rust
// Create terminal renderer
let atlas = FontAtlas::load_default(gl)?;
let terminal_grid = TerminalGrid::new(gl, atlas, (800, 600))?;

// Update cells and render
terminal_grid.update_cells(gl, cell_data.iter())?;
renderer.render(&terminal_grid);
```

### TerminalGrid
Main rendering component managing the terminal display. Handles shader programs, cell data, GPU
buffers, and rendering state. Key methods include `new()` for initialization, `update_cells()`
for content updates, and sizing queries.

### FontAtlas
Manages the 3D texture atlas containing all font glyphs. Provides character-to-texture-coordinate
mapping with fast ASCII optimization. Supports loading default or custom font atlases.


### Cell Data Structure

Each terminal cell requires:
- **symbol**: Character or grapheme to display (`&str`)
- **style**: `FontStyle` enum (Normal, Bold, Italic, BoldItalic)
- **effect**: `GlyphEffect` enum (None, Underline, Strikethrough)
- **fg/bg**: Colors as 32-bit ARGB values (`0xAARRGGBB`)

## Font Atlas 3D Texture Architecture

The font atlas uses a WebGL 3D texture where each slice contains a 4×4 grid of glyphs (16 per
slice). This provides 16× better memory utilization than one-glyph-per-layer approaches while
maintaining O(1) coordinate lookups through simple bit operations. The system supports 512 base
glyphs × 4 styles + emoji.

### 3D Texture Coordinate System

The font atlas uses a 3D texture organized as multiple slices, each containing a 4×4 grid of glyphs:

| Dimension  | Size        | Formula         | Description             |
|------------|-------------|-----------------|-------------------------|
| **Width**  | Cell × 4    | 12 × 4 = 48px   | 4 glyphs horizontally   |
| **Height** | Cell × 4    | 18 × 4 = 72px   | 4 glyphs vertically     |
| **Depth**  | ⌈Glyphs/16⌉ | Next power of 2 | One slice per 16 glyphs |

**Coordinate Mapping:**
- Glyph ID → Slice: `ID ÷ 16`
- Position in slice: `ID % 16`
- Grid coordinates: `(pos % 4, pos ÷ 4)`

This layout packs 16 glyphs per slice while maintaining fast coordinate calculation
through simple bit shifts and masks.

### Glyph ID Encoding and Mapping

#### Glyph ID Bit Layout (16-bit)

| Bit(s) | Flag Name     | Hex Mask | Binary Mask           | Description               |
|--------|---------------|----------|-----------------------|---------------------------|
| 0-8    | GLYPH_ID      | `0x01FF` | `0000_0001_1111_1111` | Base glyph identifier     |
| 9      | BOLD          | `0x0200` | `0000_0010_0000_0000` | Bold font style           |
| 10     | ITALIC        | `0x0400` | `0000_0100_0000_0000` | Italic font style         |
| 11     | EMOJI         | `0x0800` | `0000_1000_0000_0000` | Emoji character flag      |
| 12     | UNDERLINE     | `0x1000` | `0001_0000_0000_0000` | Underline effect          |
| 13     | STRIKETHROUGH | `0x2000` | `0010_0000_0000_0000` | Strikethrough effect      |
| 14-15  | RESERVED      | `0xC000` | `1100_0000_0000_0000` | Reserved for future use   |

#### ID to 3D Position Examples

| Character | Style       | Glyph ID | Calculation            | Result                | 
|-----------|-------------|----------|------------------------|-----------------------|
| ' ' (32)  | Normal      | 0x0020   | 32÷16=2, 32%16=0       | Slice 2, Grid (0,0)   |
| 'A' (65)  | Normal      | 0x0041   | 65÷16=4, 65%16=1       | Slice 4, Grid (1,0)   |
| 'A' (65)  | Bold+Italic | 0x0641   | 1601÷16=100, 1601%16=1 | Slice 100, Grid (1,0) |
| '€'       | Normal      | 0x0080   | Mapped to ID 128       | Slice 8, Grid (0,0)   |
| '🚀'      | Emoji       | 0x0881   | With emoji bit set     | Slice 136, Grid (1,0) |

The consistent modular arithmetic ensures that style variants maintain the same grid position
within their respective slices, improving texture cache coherence.

### ASCII Optimization

ASCII characters (0-127) bypass the HashMap lookup entirely through direct bit manipulation.
For ASCII input, the glyph ID is computed as `char_code | style_bits`, providing zero-overhead
character mapping. Non-ASCII characters use a HashMap for flexible Unicode suppor. This approach
optimizes for the common case (>95% ASCII in typical terminal content) while maintaining full
Unicode capability.

## GPU Buffer Architecture

The renderer uses five specialized buffers managed through a Vertex Array Object (VAO) to
achieve single-draw-call rendering. Each buffer serves a specific purpose in the instanced
rendering pipeline, with careful attention to memory alignment and update patterns.

### Buffer Layout Summary

| Buffer                | Type | Size         | Usage          | Update Freq | Purpose             |
|-----------------------|------|--------------|----------------|-------------|---------------------|
| **Vertex**            | VBO  | 64 bytes     | `STATIC_DRAW`  | Never       | Quad geometry       |
| **Index**             | IBO  | 6 bytes      | `STATIC_DRAW`  | Never       | Triangle indices    |
| **Instance Position** | VBO  | 4 bytes/cell | `STATIC_DRAW`  | On resize   | Grid coordinates    |
| **Instance Cell**     | VBO  | 8 bytes/cell | `DYNAMIC_DRAW` | Per frame   | Glyph ID + colors   |
| **Uniform**           | UBO  | 80 bytes     | `DYNAMIC_DRAW` | On resize   | Projection + params |

### Vertex Attribute Bindings

| Location | Attribute   | Type    | Components       | Divisor | Source Buffer     |
|----------|-------------|---------|------------------|---------|-------------------|
| 0        | Position    | `vec2`  | x, y             | 0       | Vertex            |
| 1        | TexCoord    | `vec2`  | u, v             | 0       | Vertex            |
| 2        | InstancePos | `uvec2` | grid_x, grid_y   | 1       | Instance Position |
| 3        | PackedData  | `uvec2` | glyph_id, colors | 1       | Instance Cell     |

### Instance Data Packing

The 8-byte `CellDynamic` structure is tightly packed to minimize bandwidth:

```
Byte Layout: [0][1][2][3][4][5][6][7]
             └─┬─┘ └───┬───┘ └───┬───┘
           Glyph ID   FG RGB   BG RGB
           (16-bit)  (24-bit) (24-bit)
```

This layout enables the GPU to fetch all cell data in a single 64-bit read, with the glyph
ID encoding both the texture coordinate and style information as described in the Glyph ID Bit
Layout section.

### Memory Layout and Performance

For a typical 12×18 pixel font with 2048 glyphs:

| Component            | Size      | Details                          |
|----------------------|-----------|----------------------------------|
| **3D Texture**       | ~1.7 MB   | 48×72×128 RGBA (16 glyphs/slice) |
| **Vertex Buffers**   | ~200 KB   | For 200×80 terminal              |
| **Cache Efficiency** | Good      | Common ASCII chars in 6 slices   |
| **Memory Access**    | Coalesced | 64-bit aligned instance data     |

The 4×4 grid layout ensures that adjacent terminal cells often access the same texture slice,
maximizing GPU cache hits. ASCII characters (the most common) are packed into the first 8 slices,
providing optimal memory locality for typical terminal content.

### Instance Data Packing

The 8-byte `CellDynamic` structure is tightly packed to minimize bandwidth and enable single-fetch
GPU reads. The glyph ID includes both character and style information as bit flags, while colors
are stored as 24-bit RGB values without alpha (alpha comes from the texture). This packing scheme
achieves a 2:1 compression ratio compared to naive storage while maintaining alignment for
efficient GPU access.

### Shader Pipeline

The renderer uses a two-stage shader pipeline optimized for instanced rendering:

#### Vertex Shader (`cell.vert`)
Transforms cell geometry from grid space to screen space using per-instance attributes. The shader:
- Calculates cell position by multiplying grid coordinates with cell size
- Applies orthographic projection for pixel-perfect rendering
- Passes packed instance data directly to fragment shader without unpacking

#### Fragment Shader (`cell.frag`)
Performs the core rendering logic with efficient 3D texture lookups:
- Extracts 16-bit glyph ID from packed instance data
- Computes 3D texture coordinates using modular arithmetic (ID → slice/grid position)
- Detects emoji glyphs via bit 11 for special color handling
- Blends foreground/background colors with glyph alpha for anti-aliasing

The key optimization is that all coordinate calculations use bit operations and modular arithmetic,
avoiding expensive conditionals or memory lookups in the hot path.

### Advanced Features

- **Emoji Rendering**: Bit 11 detection for full-color emoji with texture-based coloring
- **Missing Glyph Handling**: Automatic fallback to space character with debug logging


### WebGL2 Feature Dependencies

The renderer requires WebGL2 for:
- **3D Textures** (`TEXTURE_3D`, `texStorage3D`, `texSubImage3D`)
- **Instanced Rendering** (`drawElementsInstanced`, `vertexAttribDivisor`)
- **Advanced Buffers** (`UNIFORM_BUFFER`, `vertexAttribIPointer`)
- **Vertex Array Objects** (`createVertexArray`)

## Build and Deployment

### Development Setup
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install tools
cargo install wasm-pack trunk

# Development server
trunk serve

# Production build
trunk build --release
```

## Design Decisions

### Why 4×4 Grid Per Slice?

- **GPU compatibility**: 2D texture arrays and 3D textures have limited layer support across
  browsers and GPUs. Many systems can't reliably handle thousands of layers, but can handle depths in
  the hundreds.
- **Depth reduction**: Packing 16 glyphs per slice reduces a 2000+ glyph atlas from 2000+ layers to
  ~140 layers, staying within widely-supported limits
- **Cache efficiency**: Related glyphs (e.g., ASCII characters) cluster in the same slice,
  improving texture cache hit rates
- **Simple addressing**: 16 glyphs per slice allows coordinate calculation using bit masking (ID &
  0x0F)

### Why Separate Style Encoding?

- Avoids duplicating glyph definitions
- Enables runtime style switching without texture lookups
- Maintains consistent coordinates for style variants

### Why Power-of-2 Texture Depth?

- Required by some GPU architectures
- Simplifies mipmap generation (if needed)

## Limitations

- Maximum 512 base glyphs (9-bit addressing)
- Fixed 4 style variants per glyph
- Monospace fonts only
- Single font family per atlas


## TODO
- [x] **Text Effects**: Underline, strikethrough
- [x] **Font Variants**: Bold, italic, and other font weight support
- [x] **Complete Glyph Set**: Report (e.g. via logging) when glyphs are missing from the atlas
- [x] **Emoji support**: Currently renders with only the foreground color
  
## Undecided|Lower Prio Features
- [ ] **Double Buffering**: Are there any benefits to double buffering for terminal rendering?
- [ ] **Dynamic Atlases**: Runtime glyph addition without regeneration
- [ ] **Partial Updates**: Only update changed cells instead of full grid
- [ ] **Context Loss Recovery**: Buffer architecture designed for WebGL context restoration
