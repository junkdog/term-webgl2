import { main as init, BeamtermRenderer, CellStyle, Size } from '@beamterm/renderer';

interface Theme {
    bg: number;
    fg: number;
    primary: number;
    secondary: number;
    success: number;
    error: number;
    warning: number;
}

const tokyoNight: Theme = {
    bg: 0x1a1b26,
    fg: 0xc0caf5,
    primary: 0x7aa2f7,
    secondary: 0xbb9af7,
    success: 0x9ece6a,
    error: 0xf7768e,
    warning: 0xe0af68,
};

class TerminalApp {
    private renderer: BeamtermRenderer;
    private cols: number;
    private rows: number;

    constructor(renderer: BeamtermRenderer) {
        this.renderer = renderer;
        const size = renderer.terminal_size();
        this.cols = size[0];
        this.rows = size[1];
    }

    public render(): void {
        this.clear();
        this.drawHeader();
        this.drawMenu();
        this.drawStatus();

        this.renderer.flush();
        this.renderer.render();
    }

    public resize_terminal(width_px: number, height_px: number): void {
        this.renderer.resize(width_px, height_px);
        let size = this.renderer.terminal_size()

        this.cols = size.width;
        this.rows = size.height;

        this.render();
    }

    private clear(): void {
        this.renderer.clear(tokyoNight.bg);
    }

    private drawHeader(): void {
        const title = "🚀 Beamterm + Vite + TypeScript";
        const style = new CellStyle().bold();
        const x = Math.floor((this.cols - title.length) / 2);

        this.renderer.write_text(1, x, title, style, tokyoNight.primary, tokyoNight.bg);
    }

    private drawMenu(): void {
        const menuItems = [
            { key: 'N', label: 'New', color: tokyoNight.success },
            { key: 'O', label: 'Open', color: tokyoNight.primary },
            { key: 'S', label: 'Save', color: tokyoNight.secondary },
            { key: 'Q', label: 'Quit', color: tokyoNight.error },
        ];

        let x = 2;
        const y = 3;

        menuItems.forEach(item => {
            const keyStyle = new CellStyle().bold().underline();
            const labelStyle = new CellStyle();

            this.renderer.write_text(y, x, `[${item.key}]`, keyStyle, item.color, tokyoNight.bg);
            x += 3;
            this.renderer.write_text(y, x, ` ${item.label}  `, labelStyle, tokyoNight.fg, tokyoNight.bg);
            x += item.label.length + 3;
        });
    }

    private drawStatus(): void {
        const status = `Cols: ${this.cols} | Rows: ${this.rows} | Ready`;
        const style = new CellStyle();
        const y = this.rows - 2;

        // Draw status bar background
        const bgStyle = new CellStyle();
        const bar = '─'.repeat(this.cols);
        this.renderer.write_text(y, 0, bar, bgStyle, tokyoNight.fg, tokyoNight.bg);

        // Draw status text
        const x = this.cols - status.length - 2;
        this.renderer.write_text(y, x, status, style, tokyoNight.secondary, tokyoNight.bg);
    }
}

async function main() {
    await init();

    const renderer = new BeamtermRenderer('#terminal');
    const app = new TerminalApp(renderer);

    let {width, height} = calculateCanvasSize();
    app.resize_terminal(width, height); // triggers rendering

    function animate() {
        renderer.render();
        requestAnimationFrame(animate);
    }
    animate();

    // Handle resize
    window.addEventListener('resize', () => {
        let {width, height} = calculateCanvasSize();

        const canvas = document.getElementById('terminal') as HTMLCanvasElement;
        canvas.width = width;
        canvas.height = height;

        app.resize_terminal(width, height);
    });
}

function calculateCanvasSize(): { width: number; height: number } {
    const width = window.innerWidth - 40;
    const height = window.innerHeight - 100;

    return { width, height };
}

main();