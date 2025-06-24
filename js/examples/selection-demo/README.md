# Beamterm Selection Demo

This demo showcases Beamterm's mouse selection and clipboard functionality with **both Linear and Block selection modes working correctly**. Selection is enabled by default and includes intelligent cleanup to prevent conflicts.

## Features Demonstrated

- **Mouse Selection**: Click and drag to select text with automatic clipboard copy
- **Selection Modes**: Linear (text flow) and Block (rectangular) selection - **both working!**
- **Mouse Event Logging**: See mouse events in the browser console
- **Clipboard Integration**: Automatic clipboard copy on selection
- **Smart Cleanup**: Automatic cleanup prevents conflicts when switching modes

## Running the Demo

1. Make sure you've built the WASM packages:
   ```bash
   cd ../../../
   ./scripts/build-wasm.zsh
   ```

2. Install dependencies and run:
   ```bash
   cd js/examples/selection-demo
   npm install
   npm run dev
   ```

3. Open your browser to `http://localhost:3001`


## API Usage Examples

### Enable Built-in Selection
```javascript
// Enable linear selection (default)
renderer.enableSelection(SelectionMode.Linear, true);

// Enable block selection
renderer.enableSelection(SelectionMode.Block, true);
```

### Selection Management
```javascript
// Check if there's an active selection
if (renderer.hasSelection()) {
    console.log('Text is selected');
}

// Clear the current selection
renderer.clearSelection();
```

## Controls

- **Selection Mode**: Choose between Linear and Block selection modes (Linear is default)
- **Enable Selection**: Re-enable selection if disabled (enabled by default)
- **Disable Selection**: Switch to mouse event logging only

## Console Output

The demo includes clean logging to help understand what's happening:
- 🚀 Mode changes and selection enabling
- 🧹 Cleanup operations when switching modes
- 🔧 Selection events (start, final selection, extracted text)

## Tips

- **Default Behavior**: Selection starts enabled with Linear mode
- **Instant Mode Switching**: Change the dropdown and selection mode updates immediately
- **Test Content**: Use the table for clearest demonstration of Block vs Linear behavior
- **Paste Testing**: Best way to verify what was actually selected
- **Clean Operation**: No errors or conflicts when switching modes repeatedly
- **Performance**: Smooth operation with automatic cleanup of old handlers 