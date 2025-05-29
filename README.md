## WebGL2 Terminal Renderer

A high-performance terminal text rendering system using WebGL2, designed for efficient
rendering of large terminal grids in web browsers.

## 🚀 High Performance Rendering
- **Sub-millisecond Rendering**: Renders entire terminal in a single draw call
- **Texture Arrays**: Uses WebGL2 texture arrays for efficient glyph storage
- **Optimized ASCII rendering**: ASCII set maps to texture layer without lookup
- **Unicode Compatibility**: Full support for Unicode characters and graphemes
- **Zero-Copy Buffer Updates**: Efficient dynamic buffer management for real-time updates

## WebGL2 Feature Dependencies
This renderer uses several features introduced in **WebGL2**:

- **2D Texture Arrays** (`TEXTURE_2D_ARRAY`): Essential for glyph storage with layer-based indexing
- **Uniform Buffer Objects** (`UNIFORM_BUFFER`): Efficient batch uniform updates
- **Instanced Rendering** (`drawElementsInstanced`): Single draw call for entire terminal
- **Integer Vertex Attributes** (`vertexAttribIPointer`): Efficient packed data handling
- **3D Texture Operations** (`texSubImage3D`): Dynamic glyph atlas population


## Core Components

The system is organized into three main crates:

### 1. **bitmap-font** - Font Atlas Generation
Converts TTF/OTF fonts into GPU-optimized bitmap atlases using cosmic-text. Analyzes
character sets, packs glyphs into power-of-2 textures, and exports PNG atlases with
compact binary metadata.

**Key Features:**
- Automatic cell dimension calculation based on font glyphs
- Power-of-2 texture sizing for optimal GPU performance
- Support for multiple font styles (Normal, Bold, Italic, Bold+Italic)
- Unicode grapheme clustering for complex character support
- Efficient glyph ID assignment with ASCII optimization

### 2. **font-atlas** - Shared Types & Serialization
Shared library for font atlas data structures and serialization. Provides glyph coordinates,
atlas configurations.

**Data Format:**
- Compact binary serialization with versioning
- Header validation (`0xBAB1F0A5`) and version checking
- Little-endian encoding for cross-platform compatibility
- Efficient deserialization for runtime loading

### 3. **term-renderer** - WebGL2 Renderer
High-performance WebGL2 terminal renderer targeting sub-millisecond frame times. Leverages GPU
instancing, texture arrays, and efficient state management to render the entire terminal in a
single draw call.

## Architecture Overview

The WebGL2 terminal renderer uses GPU instancing to achieve high-performance text rendering
by reusing the same quad geometry for every terminal cell while varying only the cell-specific
data (position, character, and colors) per instance.

### Coordinate System and Projection

The renderer uses an **orthographic projection** that maps directly to screen pixels:

```
Screen Space (Pixels)          Normalized Device Coordinates
┌─────────────────────┐       ┌─────────────────────┐
│ (0,0)      (w,0)    │  →    │ (-1,1)     (1,1)    │
│                     │       │                     │
│                     │       │                     │
│ (0,h)      (w,h)    │       │ (-1,-1)    (1,-1)   │
└─────────────────────┘       └─────────────────────┘
```

This ensures pixel-perfect rendering without floating-point precision issues that could cause character misalignment.

### Font Atlas Texture Array Memory Layout

#### Overall Structure
The font atlas uses a WebGL 2D Texture Array where each layer contains a single glyph. The layer index
encodes both the base glyph ID and the font style. It based on the representation of `Glyph::id`:

##### Glyph ID Bit Layout (16-bit)

| Bit(s) | Flag Name     | Hex Mask | Binary Mask           | Description               |
|--------|---------------|----------|-----------------------|---------------------------|
| 0-8    | GLYPH_ID      | `0x01FF` | `0000_0001_1111_1111` | Base glyph id             |
| 9      | BOLD          | `0x0200` | `0000_0010_0000_0000` | Bold font style           |
| 10     | ITALIC        | `0x0400` | `0000_0100_0000_0000` | Italic font style         |
| 11     | EMOJI         | `0x0800` | `0000_1000_0000_0000` | Emoji character           |
| 12     | UNDERLINE     | `0x1000` | `0001_0000_0000_0000` | Underline text effect     |
| 13     | STRIKETHROUGH | `0x2000` | `0010_0000_0000_0000` | Strikethrough text effect |
| 14-15  | RESERVED      | `0xC000` | `1100_0000_0000_0000` | Reserved for future use   |

- The first 9 bits (0-8) represent the base glyph ID, allowing for 512 unique glyphs.
- Underlined and strikethrough styles are mutually exclusive.
- Emoji glyphs implicitly clear any other style bits.
- The glyph ID is the basis for the texture array layer index in the WebGL2 shader:

```glsl
// Fragment shader texture array uses the full glyph ID,
// except UNDERLINE and STRIKETHROUGH bits
float layer = float(v_packed_data.x & (0xFFFFu ^ 0x1800u));
vec4 glyph_color = texture(u_sampler, vec3(v_tex_coord, layer));
```

#### Memory Regions by Font Style

| Layer Slice Index Range | Seq Grid Index | Glyph Type  | Description          |
|-------------------------|----------------|-------------|----------------------|
| `0x000` - `0x1FF`       | `0x0` - `0xF`  | Normal      | Base glyphs          |
| `0x200` - `0x3FF`       | `0x0` - `0xF`  | Bold        | Bold variants        |  
| `0x400` - `0x5FF`       | `0x0` - `0xF`  | Italic      | Italic variants      |
| `0x600` - `0x7FF`       | `0x0` - `0xF`  | Bold+Italic | Bold+Italic variants |
| `0x800` - `0x9FF`       | `0x0` - `0xF`  | Emoji       | Emoji variants       |

All regions contain the same glyph layout, except for where each region can pack up to 512 glyphs.

#### Character Mapping

| Character   | Style            | Binary Representation | Hex Value | Description         |
|-------------|------------------|-----------------------|-----------|---------------------|
| 'A' (0x41)  | Normal           | `0000_0000_0100_0001` | `0x0041`  | Plain 'A'           |
| 'A' (0x41)  | Bold             | `0000_0010_0100_0001` | `0x0241`  | Bold 'A'            |
| 'A' (0x41)  | Bold + Italic    | `0000_0110_0100_0001` | `0x0641`  | Bold italic 'A'     |
| 'A' (0x41)  | Bold + Underline | `0001_0010_0100_0001` | `0x0A41`  | Bold underlined 'A' |
| '🚀' (0x81) | Emoji            | `0000_1000_1000_0001` | `0x8081`  | "rocket" emoji      |

ASCII characters (0-127) map directly to the layer's base ID, allowing for fast rendering without
a lookup. Non-ASCII characters require a HashMap lookup to find their base glyph ID.

In code:

```rust
// Fast path for ASCII characters
if ch.is_ascii() {
    layer_id = (ch as i32) | style.layer_mask();
} else {
    // Slower HashMap lookup for Unicode
    layer_id = atlas.lookup(ch) | style.layer_mask();
}
```

**Grapheme Clustering**: The font generator uses Unicode segmentation to properly handle complex characters

**Missing Glyph Fallback**: When a glyph is not found, the renderer falls back to a space character (layer 0x20)
to prevent rendering artifacts.

### Rendering Pipeline

```
Font File (TTF/OTF) → Bitmap Font Generator → Font Atlas (PNG + Binary)
                                                       ↓
Terminal Data → WebGL2 Renderer → GPU Instanced Rendering → Browser Canvas
```

### Core Concept

Rather than drawing each character individually, the renderer:
1. Defines a single quad geometry (4 vertices, 2 triangles)
2. Creates instance data for each terminal cell
3. Renders the entire terminal grid in a single `drawElementsInstanced` call

This approach minimizes draw calls and GPU state changes, enabling sub-millisecond rendering
even for large terminal grids.

### Buffer Layout

```
┌─────────────────┬─────────────────┬─────────────────┬─────────────────┐
│ Vertex Buffer   │ Index Buffer    │ Instance Pos    │ Instance Cell   │
│ Quad geometry   │ Triangle order  │ Grid positions  │ Glyph + colors  │
│ (static)        │ (static)        │ (static)        │ (dynamic)       │
└─────────────────┴─────────────────┴─────────────────┴─────────────────┘
```

#### Static Buffers (Set Once)

- **Vertex Buffer**: Contains quad geometry with position and texture coordinates for 4 vertices
- **Index Buffer**: Triangle indices `[0,1,2, 0,3,1]` for rendering 2 triangles per quad
- **Instance Position Buffer**: Grid coordinates for each terminal cell ([`CellStatic`])

#### Dynamic Buffers (Updated Per Frame)

- **Instance Cell Buffer**: Character and color data for each cell ([`CellDynamic`])

#### Uniform Resources

- **Uniform Buffer Object**: Projection matrix and cell dimensions ([`CellUbo`])
- **Font Atlas Texture**: 2D texture array with one glyph per layer ([`FontAtlas`])

### Data Structure Details

#### CellStatic - Grid Positioning
```rust
#[repr(C, align(4))]
struct CellStatic {
    pub grid_xy: [u16; 2], // Grid coordinates (0-65535 range)
}
```

- **4-byte alignment** for GPU efficiency
- **Static data** uploaded once during initialization
- Covers terminal grids up to 65,535 × 65,535 cells

#### CellDynamic - Visual Content
```rust
#[repr(C, align(4))]
struct CellDynamic {
    /// Packed as: [layer_lo, layer_hi, fg_r, fg_g, fg_b, bg_r, bg_g, bg_b]
    pub data: [u8; 8],
}
```

**Color Format**: Colors are stored as **24-bit RGB** (no alpha in instance data). Alpha blending is handled in the
fragment shader based on glyph texture alpha.

#### CellUbo - Uniform Data
```rust
#[repr(C, align(16))] // std140 layout requirement
struct CellUbo {
    pub projection: [f32; 16], // 4×4 matrix (64 bytes)
    pub cell_size: [f32; 2],   // vec2 (8 bytes + 8 bytes padding)
}
```

- **std140 layout** ensures consistent memory layout across platforms
- **16-byte alignment** required for uniform buffer objects
- Total size: 80 bytes (64 + 8 + 8 padding)

### Shader Pipeline

#### Vertex Shader (`cell.vert`)

**Per-Vertex Attributes** (4 times per cell):
- `a_pos` (location 0): Vertex position within the quad
- `a_tex_coord` (location 1): Texture coordinates (0,0) to (1,1)

**Per-Instance Attributes** (1 time per cell):
- `a_instance_pos` (location 2): Grid position (x,y) in cell units
- `a_packed_data` (location 3): Glyph layer ID and packed color data

#### Fragment Shader (`cell.frag`)

**Input**:
- Interpolated texture coordinates (`v_tex_coord`)
- Flat instance data (`v_packed_data`)

### Memory Efficiency

#### Storage Requirements

For a 200×80 terminal (16,000 cells):

| Component     | Size per Cell ( | Total Size | Usage Pattern      |
|---------------|-----------------|------------|--------------------|
| CellStatic    | 4 bytes         | 64 KB      | Initialization     |
| CellDynamic   | 8 bytes         | 128 KB     | Updated per frame  |
| Vertex Buffer | —               | 64 bytes   | Static quad        |
| Index Buffer  | —               | 6 bytes    | Static indices     |
| UBO           | —               | 80 bytes   | Initialization     |

- **Total Dynamic Memory**: ~192 KB per terminal update
- **Static Memory**: ~64 KB (allocated once)

### Update Patterns

| Component            | Update Frequency | Trigger                |
|----------------------|------------------|------------------------|
| Vertex/Index Buffers | Once             | Initialization         |
| Instance Position    | Once             | Terminal resize        |
| Instance Cell Buffer | Per frame        | Content changes        |
| Uniform Buffer       | Per resize       | Window/terminal resize |
| Font Atlas           | Once             | Font loading           |

### Error Handling and Debugging

```rust
enum Error {
    Initialization(String), // Canvas/WebGL setup failures
    Shader(String),         // Compilation/linking errors  
    Resource(String),       // Buffer/texture creation failures
    Data(String),           // Font loading/parsing errors
}
```

### Build and Deployment

#### Development Setup
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

## TODO
- [ ] **Text Effects**: Underline, strikethrough
- [x] **Font Variants**: Bold, italic, and other font weight support
- [ ] **Complete Glyph Set**: Report (e.g. via logging) when glyphs are missing from the atlas
- [ ] **Emoji support**: Currently renders with only the foreground color
  
## Undecided Features
- [ ] **Double Buffering**: Are there any benefits to double buffering for terminal rendering?
- [ ] **Dynamic Atlases**: Runtime glyph addition without regeneration
- [ ] **Partial Updates**: Only update changed cells instead of full grid
