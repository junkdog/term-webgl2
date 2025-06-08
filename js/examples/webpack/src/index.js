// js/examples/webpack/src/index.js
import { main as init, BeamtermRenderer, CellStyle, JsCellData, JsSpan } from '@beamterm/renderer';

async function main() {
    await init();

    const renderer = new BeamtermRenderer('#terminal');
    const [cols, rows] = renderer.terminal_size();

    console.log(`Terminal size: ${cols}x${rows}`);

    // Create a simple UI
    drawBorder(renderer);
    drawTitle(renderer);
    drawContent(renderer);

    renderer.flush();
    renderer.render();
}

function drawBorder(renderer) {
    const [cols, rows] = renderer.terminal_size();
    const style = new CellStyle();
    const borderColor = 0x414868;
    const bgColor = 0x1a1b26;

    // Top and bottom borders
    for (let x = 0; x < cols; x++) {
        renderer.update_cell(0, x, new JsCellData('─', style, borderColor, bgColor));
        renderer.update_cell(rows - 1, x, new JsCellData('─', style, borderColor, bgColor));
    }

    // Left and right borders
    for (let y = 0; y < rows; y++) {
        renderer.update_cell(y, 0, new JsCellData('│', style, borderColor, bgColor));
        renderer.update_cell(y, cols - 1, new JsCellData('│', style, borderColor, bgColor));
    }

    // Corners
    const corners = [
        { pos: [0, 0], char: '┌' },
        { pos: [0, cols - 1], char: '┐' },
        { pos: [rows - 1, 0], char: '└' },
        { pos: [rows - 1, cols - 1], char: '┘' }
    ];

    corners.forEach(({ pos: [y, x], char }) => {
        renderer.update_cell(y, x, new JsCellData(char, style, borderColor, bgColor))
    });
}

function drawTitle(renderer) {
    const title = " BeamTERM Webpack Example ";
    const style = new CellStyle().bold();
    const [cols] = renderer.terminal_size();
    const startX = Math.floor((cols - title.length) / 2);

    renderer.write_text(0, startX, title, style, 0x7aa2f7, 0x1a1b26);
}

function drawContent(renderer) {
    // First render the simple content lines
    const content = [
        { text: "Welcome to BeamTERM!", style: new CellStyle().bold(), color: 0x9ece6a },
        { text: "", style: new CellStyle(), color: 0xc0caf5 },
        { text: "Features:", style: new CellStyle().underline(), color: 0x7aa2f7 },
        { text: "• Sub-millisecond rendering", style: new CellStyle(), color: 0xc0caf5 },
        { text: "• Full Unicode and (tiny) emoji support 🙂", style: new CellStyle(), color: 0xc0caf5 },
    ];

    content.forEach((line, index) => {
        renderer.write_text(3 + index, 3, line.text, line.style, line.color, 0x1a1b26);
    });

    // For the last line, demonstrate multiple font styles
    const lastLineY = 3 + content.length;
    let x = 3;
    const color = 0xc0caf5;
    const bg = 0x1a1b26;

    // Write each part with different styles
    const parts = [
        { text: "• ", style: new CellStyle() },
        { text: "Multiple", style: new CellStyle().bold() },
        { text: " ", style: new CellStyle() },
        { text: "font", style: new CellStyle().italic() },
        { text: " ", style: new CellStyle() },
        { text: "styles", style: new CellStyle().bold().italic() },
    ];

    parts.forEach(part => {
        renderer.write_text(lastLineY, x, part.text, part.style, color, bg);
        x += part.text.length;
    });

    // WebGL2 powered line
    renderer.write_text(lastLineY + 1, 3, "• WebGL2 powered", new CellStyle(), 0xc0caf5, 0x1a1b26);
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', main);
} else {
    main();
}