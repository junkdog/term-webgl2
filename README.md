## WebGL2 Terminal Renderer

A high-performance terminal text rendering system using WebGL2, designed for efficient
rendering of large terminal grids in web browsers.


### Core Components

The system is organized into three main crates:

#### 1. **bitmap-font** - Font Atlas Generation
Converts TTF/OTF fonts into GPU-optimized bitmap atlases using cosmic-text. Analyzes
character sets, packs glyphs into power-of-2 textures, and exports PNG atlases with
compact binary metadata.

#### 2. **font-atlas** - Shared Types & Serialization
Shared library for font atlas data structures and serialization. Provides glyph coordinates,
atlas configurations.

#### 3. **web** - WebGL2 Renderer
High-performance WebGL2 terminal renderer targeting sub-millisecond frame times. Leverages GPU
instancing, texture arrays, and efficient state management to render entire terminal grids in
a single draw call.

### Rendering Pipeline

```
Font File (TTF/OTF) → Bitmap Font Generator → Font Atlas (PNG + Binary)
                                                       ↓
Terminal Data → WebGL2 Renderer → GPU Instanced Rendering → Browser Canvas
```

## Key Features

### 🚀 High Performance Rendering
- **GPU Instancing**: Renders entire terminal in a single draw call
- **Texture Arrays**: Uses WebGL2 texture arrays for efficient glyph storage
- **Optimized ASCII rendering**: ASCII set maps to texture layer without lookup
- **Uniform Buffer Objects**: Minimizes CPU-GPU communication overhead

### 📝 Advanced Text Support
- **Unicode Compatibility**: Full support for Unicode characters and graphemes
- **Monospace Optimized**: Designed specifically for terminal/console applications
- **Custom Glyph Sets**: Configurable character sets for specific use cases
- **Fallback Handling**: Graceful handling of missing glyphs

## Technical Implementation

### WebGL2 Shaders

**Shader Features:**
- Instanced rendering with per-cell positioning
- Uniform buffer objects for projection and cell sizing
- Efficient attribute packing to minimize bandwidth
- Texture array with one glyph per array layer

### Memory Layout

**Cell Data Packing:**
```rust
// packed into 8 bytes per cell, GPU-optimized layout
#[repr(C, align(4))]
struct CellDynamic {
    pub data: [u8; 8], // 2b layer, fg:rgb, bg:rgb
}
```

### Performance Characteristics
- **Sub-millisecond Rendering**: Designed to use ≤1ms per frame for terminal grids up to 200x80 characters on
                                 mid-range hardware
- **Consistent Frame Times**: Predictable performance without frame spikes or stuttering

## TODO
- [ ] **Text Effects**: Underline, strikethrough
- [ ] **Font Variants**: Bold, italic, and other font weight support
  
## Undecided Features
- [ ] **Dynamic Atlases**: Runtime glyph addition without regeneration
- [ ] **Partial Updates**: Only update changed cells instead of full grid
