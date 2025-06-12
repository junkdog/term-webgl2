import { main as init, BeamtermRenderer, CellStyle, Cell } from '@beamterm/renderer';

// Initialize and run the terminal demo
async function main() {
    try {
        // Initialize WASM module
        await init();
        console.log('✅ WASM module initialized');

        // Create renderer instance
        const renderer = new BeamtermRenderer('#terminal');
        const size = renderer.terminalSize();
        console.log(`✅ Terminal created: ${size.width}x${size.height}`);

        // Create demo app
        const app = new TerminalDemo(renderer);

        // Initial render
        app.render();

        // Start animation loop
        app.startAnimation();

        // Handle window resize
        window.addEventListener('resize', () => {
            const canvas = document.getElementById('terminal');
            canvas.width = window.innerWidth - 40;
            canvas.height = window.innerHeight - 40;

            renderer.resize(canvas.width, canvas.height);
            app.updateSize();
            app.render();
        });

    } catch (error) {
        console.error('❌ Failed to initialize:', error);
        document.body.innerHTML = `
            <div style="color: red; padding: 20px;">
                <h2>Failed to initialize Beamterm</h2>
                <pre>${error.message}</pre>
                <p>Make sure WASM is built: <code>./build.zsh build-wasm</code></p>
            </div>
        `;
    }
}

// Terminal demo application
class TerminalDemo {
    constructor(renderer) {
        this.renderer = renderer;
        this.frame = 0;
        this.size = renderer.terminalSize();
    }

    updateSize() {
        this.size = this.renderer.terminalSize();
    }

    render() {
        // Create a new batch for this frame
        const batch = this.renderer.batch();

        // Clear the terminal
        batch.clear(0x1a1b26);

        // Draw all UI elements
        this.drawBorder(batch);
        this.drawHeader(batch);
        this.drawContent(batch);
        this.drawColorPalette(batch);
        this.drawStatusBar(batch);

        // Synchronize all updates to GPU in one call
        batch.flush();

        // Render the frame
        this.renderer.render();
    }

    drawBorder(batch) {
        const style = new CellStyle();
        const borderColor = 0x414868;
        const bgColor = 0x1a1b26;

        // Collect all border cells for efficient batch update
        const borderCells = [];

        // Top and bottom borders
        for (let x = 0; x < this.size.width; x++) {
            borderCells.push([x, 0, { symbol: '─', style: style.bits, fg: borderColor, bg: bgColor }]);
            borderCells.push([x, this.size.height - 1, { symbol: '─', style: style.bits, fg: borderColor, bg: bgColor }]);
        }

        // Left and right borders
        for (let y = 0; y < this.size.height; y++) {
            borderCells.push([0, y, { symbol: '│', style: style.bits, fg: borderColor, bg: bgColor }]);
            borderCells.push([this.size.width - 1, y, { symbol: '│', style: style.bits, fg: borderColor, bg: bgColor }]);
        }

        // Corners
        const corners = [
            { pos: [0, 0], char: '┌' },
            { pos: [this.size.width - 1, 0], char: '┐' },
            { pos: [0, this.size.height - 1], char: '└' },
            { pos: [this.size.width - 1, this.size.height - 1], char: '┘' }
        ];

        corners.forEach(({ pos: [x, y], char }) => {
            borderCells.push([x, y, { symbol: char, style: style.bits, fg: borderColor, bg: bgColor }]);
        });

        // Update all border cells in one efficient call
        batch.putCells(borderCells);
    }

    drawHeader(batch) {
        const title = " 🚀 BeamTERM Webpack Example ";
        const style = new CellStyle().bold();
        const startX = Math.floor((this.size.width - title.length) / 2);

        batch.writeText(startX, 0, title, style, 0x7aa2f7, 0x1a1b26);

        // Version info
        const version = "v0.1.0";
        batch.writeText(this.size.width - version.length - 2, 0, version, new CellStyle(), 0x565f89, 0x1a1b26);
    }

    drawContent(batch) {
        // Welcome message with different styles
        const content = [
            { text: "Welcome to BeamTERM!", y: 3, style: new CellStyle().bold(), color: 0x9ece6a },
            { text: "High-performance WebGL2 terminal renderer", y: 5, style: new CellStyle(), color: 0xc0caf5 },
        ];

        content.forEach(({ text, y, style, color }) => {
            const x = Math.floor((this.size.width - text.length) / 2);
            batch.writeText(x, y, text, style, color, 0x1a1b26);
        });

        // Feature list
        const features = [
            "• GPU-accelerated text rendering",
            "• Multiple text styles",
            "• (Tiny) emoji support 🙂",
            "• Zero allocations in render loop",
            "• Experimental TS/JS API",
        ];

        features.forEach((feature, i) => {
            batch.writeText(4, 7 + i, feature, new CellStyle(), 0xa9b1d6, 0x1a1b26);
        });

        // Style demonstrations
        this.drawStyleDemo(batch, 14);
    }

    drawStyleDemo(batch, startY) {
        batch.writeText(4, startY, "Text Styles:", new CellStyle().bold(), 0xc0caf5, 0x1a1b26);

        const styleDemo = [
            { text: "Normal", style: new CellStyle(), x: 4 },
            { text: "Bold", style: new CellStyle().bold(), x: 14 },
            { text: "Italic", style: new CellStyle().italic(), x: 22 },
            { text: "Underline", style: new CellStyle().underline(), x: 32 },
            { text: "Strikethrough", style: new CellStyle().strikethrough(), x: 45 },
        ];

        styleDemo.forEach(({ text, style, x }) => {
            batch.writeText(x, startY + 2, text, style, 0x7aa2f7, 0x1a1b26);
        });

        // Combined styles
        const combined = new CellStyle().bold().italic().underline();
        batch.writeText(4, startY + 4, "Combined: Bold + Italic + Underline", combined, 0xbb9af7, 0x1a1b26);
    }

    drawColorPalette(batch) {
        const startY = 20;
        const colors = [
            { name: "Red", value: 0xf7768e },
            { name: "Green", value: 0x9ece6a },
            { name: "Blue", value: 0x7aa2f7 },
            { name: "Purple", value: 0xbb9af7 },
            { name: "Orange", value: 0xe0af68 },
            { name: "Cyan", value: 0x7dcfff },
        ];

        batch.writeText(4, startY, "Color Palette:", new CellStyle().bold(), 0xc0caf5, 0x1a1b26);

        // Collect all color block cells for batch update
        const colorCells = [];

        colors.forEach((color, i) => {
            const x = 4 + (i % 3) * 20;
            const y = startY + 2 + Math.floor(i / 3) * 2;

            // Color blocks (5 cells wide for better visibility)
            for (let j = 0; j < 5; j++) {
                colorCells.push([x + j, y, {
                    symbol: '█',
                    style: new CellStyle().bits,
                    fg: color.value,
                    bg: 0x1a1b26
                }]);
            }

            // Color name
            batch.writeText(x + 6, y, color.name, new CellStyle(), 0xa9b1d6, 0x1a1b26);
        });

        // Update all color cells at once for efficiency
        batch.putCells(colorCells);
    }

    drawStatusBar(batch) {
        const y = this.size.height - 2;

        // Status bar background
        const statusBg = [];
        for (let x = 1; x < this.size.width - 1; x++) {
            statusBg.push([x, y, { symbol: ' ', style: 0, fg: 0xc0caf5, bg: 0x24283b }]);
        }
        batch.putCells(statusBg);

        // Status text
        const fps = Math.round(1000 / 16); // Approximate FPS
        const status = ` FPS: ${fps} | Cells: ${this.size.width * this.size.height} | Frame: ${this.frame} `;
        batch.writeText(2, y, status, new CellStyle(), 0xc0caf5, 0x24283b);

        // Right-aligned info
        const info = "Press F11 for fullscreen ";
        batch.writeText(this.size.width - info.length - 2, y, info, new CellStyle(), 0xa9b1d6, 0x24283b);
    }

    startAnimation() {
        const animate = () => {
            const batch = this.renderer.batch();

            // Animated spinner
            const spinnerChars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            const spinnerChar = spinnerChars[this.frame % spinnerChars.length];
            batch.putCell(this.size.width - 4, this.size.height - 2, new Cell(spinnerChar, new CellStyle(), 0x7aa2f7, 0x24283b));

            // Animated wave effect
            this.drawWaveAnimation(batch);

            // Update frame counter
            this.frame++;

            // Only re-render the full screen every 60 frames
            if (this.frame % 60 === 0) {
                this.render();
            } else {
                // Otherwise just update the animated parts
                batch.flush();
                this.renderer.render();
            }

            requestAnimationFrame(animate);
        };

        animate();
    }

    drawWaveAnimation(batch) {
        const waveY = 26;
        const waveChars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█', '▇', '▆', '▅', '▄', '▃', '▂'];
        const waveColors = [0x7aa2f7, 0x7dcfff, 0x89ddff, 0x7dcfff, 0x7aa2f7];

        const waveCells = [];

        for (let x = 1; x < this.size.width - 1; x++) {
            const offset = (this.frame / 2 + x * 2) % waveChars.length;
            const charIndex = Math.floor(offset) % waveChars.length;
            const colorIndex = Math.floor((this.frame / 10 + x) % waveColors.length);

            waveCells.push([x , waveY, {
                symbol: waveChars[charIndex],
                style: new CellStyle().bits,
                fg: waveColors[colorIndex],
                bg: 0x1a1b26
            }]);
        }

        batch.putCells(waveCells);
    }
}

// Start the application when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', main);
} else {
    main();
}