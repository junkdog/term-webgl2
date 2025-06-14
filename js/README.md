# @beamterm/renderer

[![npm version](https://img.shields.io/npm/v/@beamterm/renderer.svg)](https://www.npmjs.com/package/@beamterm/renderer)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance WebGL2 terminal renderer achieving sub-millisecond render times through GPU-accelerated instanced rendering.

## ✨ Features

- **📦 Zero Dependencies**: Pure WASM + WebGL2, no external runtime dependencies
- **🎨 Rich Text Styling**: Bold, italic, underline, strikethrough with full color support
- **⚡ Efficient Updates**: Batch cell updates with single GPU buffer upload
- **📐 Responsive**: Automatic terminal resizing with proper aspect ratio maintenance
- **🎯 TypeScript Ready**: Full TypeScript definitions included

## 📋 Requirements

- **WebGL2** capable browser
- **WASM** support

Should work with any modern browser.

## 📦 Installation

### NPM/Yarn (NOT YET RELEASED)


```bash
npm install @beamterm/renderer
# or
yarn add @beamterm/renderer
```

### CDN

```html
<script type="module">
import Beamterm from 'https://unpkg.com/@beamterm/renderer@latest/dist/cdn/beamterm.min.js';
</script>
```

## 🚀 Quick Start

### ES Modules (Recommended)

```javascript
import { main as init, style, BeamtermRenderer } from '@beamterm/renderer';

// Initialize WASM module
await init();

// Create renderer attached to canvas
const renderer = new BeamtermRenderer('#terminal');

// Get terminal dimensions
const size = renderer.terminalSize();
console.log(`Terminal: ${size.width}×${size.height} cells`);

// Create a batch for efficient updates
const batch = renderer.batch();

// Clear terminal with background color
batch.clear(0x1a1b26);

// Write styled text
const style = style().bold().underline();
batch.text(2, 1, "Hello, Beamterm!", style.fg(0x7aa2f7).bg(0x1a1b26));

// Draw a box
const boxStyle = new CellStyle();
batch.text(1, 0, "┌──────────────────┐", boxStyle.fg(0x565f89).bg(0x1a1b26));
batch.text(1, 2, "└──────────────────┘", boxStyle.fg(0x565f89).bg(0x1a1b26));

// Update individual cells
batch.cell(0, 0, { 
  symbol: "🚀", 
  style: style().underline(),
  fg: 0xffffff, 
  bg: 0x1a1b26 
});

// Flush all updates to GPU
batch.flush();

// Render frame
renderer.render();
```

### TypeScript

```typescript
import { main as init, style, BeamtermRenderer, Batch } from '@beamterm/renderer';

async function createTerminal(): Promise<void> {
  await init();
  
  const renderer = new BeamtermRenderer('#terminal');
  const batch: Batch = renderer.batch();
  
  // TypeScript provides full type safety
  const labelStyle = labelStyle()
    .bold()
    .italic()
    .underline()
    .fg(0x9ece6a)
    .bg(0x1a1b26);
    
  batch.text(0, 0, "TypeScript Ready! ✨", labelStyle);
  batch.flush();
  renderer.render();
}
```

## 📖 API Reference

### BeamtermRenderer

The main renderer class that manages the WebGL2 context and rendering pipeline.

```javascript
const renderer = new BeamtermRenderer(canvasSelector);
```

#### Methods

- **`batch()`**: Create a new batch for efficient cell updates
- **`render()`**: Render the current frame to the canvas
- **`resize(width, height)`**: Resize the canvas and recalculate terminal dimensions
- **`terminalSize()`**: Get terminal dimensions as `{ width, height }` in cells
- **`cellSize()`**: Get cell dimensions as `{ width, height }` in pixels

### Batch

Batch operations for efficient GPU updates. All cell modifications should go through a batch.

```javascript
const batch = renderer.batch();
```

#### Methods

- **`clear(backgroundColor)`**: Clear entire terminal with specified color
- **`cell(x, y, cell)`**: Update a single cell
- **`cells(cellArray)`**: Update multiple cells (array of `[x, y, cell]`)
- **`text(x, y, text, style)`**: Write text starting at position
- **`fill(x, y, width, height, cell)`**: Fill rectangular region
- **`flush()`**: Upload all changes to GPU (required before render)

### CellStyle

Fluent API for text styling.

```javascript
const myStyle = new style()
  .bold()
  .italic()
  .underline()
  .strikethrough()
  .fg(0x7aa2f7)
  .bg(0x204060)
```

### Cell Data Structure

```javascript
{
  symbol: string,    // Single character or emoji
  style: number,     // Style bits or CellStyle.bits
  fg: number,        // Foreground color (0xRRGGBB)
  bg: number         // Background color (0xRRGGBB)
}
```

### Color Format

Colors are 24-bit RGB values in hex format:

```javascript
const white = 0xffffff;
const black = 0x000000;
const red = 0xff0000;
const tokyoNightBg = 0x1a1b26;
```

## 🎯 Common Patterns

### Animation Loop

```javascript
function animate() {
  const batch = renderer.batch();
  
  // Update terminal content
  batch.clear(0x1a1b26);
  batch.text(0, 0, `Frame: ${Date.now()}`, style().bg(0x1a1b26));
  
  // Flush and render
  batch.flush();
  renderer.render();
  
  requestAnimationFrame(animate);
}
```

### Responsive Terminal

```javascript
window.addEventListener('resize', () => {
  const canvas = document.getElementById('terminal');
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  
  renderer.resize(canvas.width, canvas.height);
  redrawTerminal();
});
```

### Efficient Mass Updates

```javascript
// Prepare all cells
const cells = [];
for (let y = 0; y < height; y++) {
  for (let x = 0; x < width; x++) {
    cells.push([x, y, {
      symbol: data[y][x],
      style: 0,
      fg: 0xffffff,
      bg: 0x000000
    }]);
  }
}

// Single batch update
batch.cells(cells);
batch.flush();
renderer.render();
```

## 🎮 Examples

Check out the [`examples/`](https://github.com/junkdog/beamterm/tree/main/js/examples) directory for complete examples:

- **[Webpack Example](examples/webpack/)** - Classic bundler setup
- **[Vite + TypeScript Example](examples/vite/)** - Modern development with HMR

## 📄 License

MIT License - see [LICENSE](https://github.com/junkdog/beamterm/blob/main/LICENSE) for details.

